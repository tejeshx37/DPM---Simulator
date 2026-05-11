#![allow(dead_code)]

use bytemuck::{Pod, Zeroable};
use std::borrow::Cow;
use wgpu::util::DeviceExt;

// ── GPU-side data structures ─────────────────────────────────────────────────
// These must match the struct layouts in update_elements.wgsl and compute.wgsl.

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct GpuNode {
    pub initial_position: [f32; 4],
    pub position:         [f32; 4],
    pub velocity:         [f32; 4],
    pub mass:             f32,
    pub _padding:         [f32; 3],
}

/// Element buffer shared between both compute passes.
/// Pass 1 (update_elements.wgsl) writes stress_col*, strain_energy_bits, is_broken.
/// Pass 2 (compute.wgsl) reads them for force computation.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct GpuElement {
    pub node_indices:        [u32; 4],
    pub stress_col0:         [f32; 4],
    pub stress_col1:         [f32; 4],
    pub stress_col2:         [f32; 4],
    pub is_broken:           u32,
    pub strain_energy_bits:  u32,  // bitcast<u32>(f32 strain_energy), written by Pass 1
    pub is_inverted:         u32,
    pub _padding:            u32,
}

/// Per-element force output from Pass 2 (compute.wgsl).
/// 4 force vectors (for nodes a, b, c, d), each as [f32; 4] (w unused).
/// CPU scatters these into per-node totals — no overflow, no atomics.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct GpuElementForces {
    pub force_a: [f32; 4],
    pub force_b: [f32; 4],
    pub force_c: [f32; 4],
    pub force_d: [f32; 4],
}

/// Material parameters passed as a uniform buffer to Pass 1.
/// `material_type` 0 = isotropic, 1 = orthotropic.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct GpuMaterial {
    // Shared
    pub density:               f32,
    pub damping:               f32,
    pub failure_strain_energy: f32,
    pub failure_tensile:       f32,
    pub failure_compressive:   f32,
    pub material_type:         u32,  // 0 = isotropic, 1 = orthotropic
    // Isotropic
    pub elasticity_modulus:    f32,
    pub poissons_ratio:        f32,
    // Orthotropic
    pub ex: f32, pub ey: f32, pub ez: f32,
    pub nu_xy: f32, pub nu_yx: f32,
    pub nu_yz: f32, pub nu_zy: f32,
    pub nu_zx: f32, pub nu_xz: f32,
    pub g_xy: f32, pub g_yz: f32, pub g_zx: f32,
    pub _pad: [f32; 3],
}

/// Per-element results read back to the CPU after both passes.
#[derive(Clone, Debug, Default)]
pub struct GpuElementResult {
    /// Updated stress matrix columns: stress[col][row_component]
    pub stress: [[f32; 3]; 3],
    /// Strain energy density for this element
    pub strain_energy: f32,
    /// Whether this element was determined to be broken by Pass 1
    pub is_broken: bool,
    /// Whether this element is inverted (det F <= 0)
    pub is_inverted: bool,
}

// ── Two-pass compute pipeline ────────────────────────────────────────────────

pub struct ComputePipeline {
    device:          wgpu::Device,
    queue:           wgpu::Queue,
    // Pass 1 — element state update (strain → stress → broken)
    update_pipeline: wgpu::ComputePipeline,
    update_bgl:      wgpu::BindGroupLayout,
    // Pass 2 — per-element force computation (no atomics)
    force_pipeline:  wgpu::ComputePipeline,
    force_bgl:       wgpu::BindGroupLayout,
}

/// A persistent session for one simulation instance, caching GPU buffers and bind groups.
pub struct GpuSession {
    pub pipeline:        std::sync::Arc<ComputePipeline>,
    nodes_buf:           wgpu::Buffer,
    elements_buf:        wgpu::Buffer,
    material_buf:        wgpu::Buffer,
    elem_forces_buf:     wgpu::Buffer,
    elements_staging:    wgpu::Buffer,
    elem_forces_staging: wgpu::Buffer,
    update_bg:           wgpu::BindGroup,
    force_bg:            wgpu::BindGroup,
    num_nodes:           usize,
    num_elements:        usize,
}

impl std::fmt::Debug for GpuSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GpuSession")
            .field("num_nodes", &self.num_nodes)
            .field("num_elements", &self.num_elements)
            .finish()
    }
}

impl std::fmt::Debug for ComputePipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComputePipeline").finish_non_exhaustive()
    }
}

unsafe impl Send for ComputePipeline {}
unsafe impl Sync for ComputePipeline {}

impl ComputePipeline {
    pub async fn new() -> Option<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::util::backend_bits_from_env().unwrap_or_else(|| {
                if cfg!(target_os = "linux") {
                    wgpu::Backends::VULKAN
                } else {
                    wgpu::Backends::PRIMARY
                }
            }),
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .ok()?;

        // ── Pass 1 shader & pipeline ─────────────────────────────────────────
        let update_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label:  Some("Update Elements Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
                include_str!("update_elements.wgsl"),
            )),
        });

        let update_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label:   Some("Update Elements BGL"),
            entries: &[
                // binding 0 — nodes (read-only)
                wgpu::BindGroupLayoutEntry {
                    binding: 0, visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false, min_binding_size: None,
                    },
                    count: None,
                },
                // binding 1 — elements (read-write, Pass 1 writes stress/broken)
                wgpu::BindGroupLayoutEntry {
                    binding: 1, visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false, min_binding_size: None,
                    },
                    count: None,
                },
                // binding 2 — material uniform
                wgpu::BindGroupLayoutEntry {
                    binding: 2, visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false, min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let update_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Update Elements PL"),
            bind_group_layouts: &[&update_bgl], push_constant_ranges: &[],
        });
        let update_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Update Elements Pipeline"), layout: Some(&update_pl),
            module: &update_shader, entry_point: "main",
        });

        // ── Pass 2 shader & pipeline ─────────────────────────────────────────
        let force_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label:  Some("Force Computation Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("compute.wgsl"))),
        });

        // Bindings: nodes(read), elements(read, stress from Pass 1), element_forces(write)
        let force_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label:   Some("Force Computation BGL"),
            entries: &[
                // binding 0 — nodes (read-only)
                wgpu::BindGroupLayoutEntry {
                    binding: 0, visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false, min_binding_size: None,
                    },
                    count: None,
                },
                // binding 1 — elements (read-only for Pass 2)
                wgpu::BindGroupLayoutEntry {
                    binding: 1, visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false, min_binding_size: None,
                    },
                    count: None,
                },
                // binding 2 — per-element forces output (write)
                wgpu::BindGroupLayoutEntry {
                    binding: 2, visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false, min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let force_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Force Computation PL"),
            bind_group_layouts: &[&force_bgl], push_constant_ranges: &[],
        });
        let force_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Force Computation Pipeline"), layout: Some(&force_pl),
            module: &force_shader, entry_point: "main",
        });

        Some(Self { device, queue, update_pipeline, update_bgl, force_pipeline, force_bgl })
    }

    pub fn create_session(
        self: std::sync::Arc<Self>,
        num_nodes: usize,
        num_elements: usize,
    ) -> GpuSession {
        let device = &self.device;

        let nodes_size = (num_nodes * std::mem::size_of::<GpuNode>()) as wgpu::BufferAddress;
        let nodes_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Nodes"),
            size:  nodes_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let elements_size = (num_elements * std::mem::size_of::<GpuElement>()) as wgpu::BufferAddress;
        let elements_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Elements"),
            size:  elements_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let material_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Material"),
            size:  std::mem::size_of::<GpuMaterial>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let elem_forces_size = (num_elements * std::mem::size_of::<GpuElementForces>()) as wgpu::BufferAddress;
        let elem_forces_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Element Forces"),
            size:  elem_forces_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let elements_staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Elements Staging"),
            size:  elements_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let elem_forces_staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Element Forces Staging"),
            size:  elem_forces_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let update_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Update Elements BG"),
            layout: &self.update_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: nodes_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: elements_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: material_buf.as_entire_binding() },
            ],
        });

        let force_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Force Computation BG"),
            layout: &self.force_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: nodes_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: elements_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: elem_forces_buf.as_entire_binding() },
            ],
        });

        GpuSession {
            pipeline: self,
            nodes_buf,
            elements_buf,
            material_buf,
            elem_forces_buf,
            elements_staging,
            elem_forces_staging,
            update_bg,
            force_bg,
            num_nodes,
            num_elements,
        }
    }

    /// Run both GPU passes in one submit.
    ///
    /// Returns:
    /// - `Vec<[f32; 3]>` — accumulated force vector per node (no overflow, pure f32)
    /// - `Vec<GpuElementResult>` — updated stress, strain energy, broken flag per element
    pub fn execute(
        &self,
        num_nodes:    usize,
        gpu_nodes:    &[GpuNode],
        gpu_elements: &[GpuElement],
        material:     &GpuMaterial,
        read_elements: bool,
    ) -> (Vec<[f32; 3]>, Vec<GpuElementResult>) {
        let n_elements = gpu_elements.len();

        // ── GPU buffers ───────────────────────────────────────────────────────
        let nodes_buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Nodes"), contents: bytemuck::cast_slice(gpu_nodes),
            usage: wgpu::BufferUsages::STORAGE,
        });

        // Elements: read-write (Pass 1 writes), readable (Pass 2 reads), copy-src (readback)
        let elements_buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Elements"), contents: bytemuck::cast_slice(gpu_elements),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });

        let material_buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Material"), contents: bytemuck::bytes_of(material),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        // Per-element force output: 4 × vec4<f32> per element = 64 bytes each
        let elem_forces_size =
            (n_elements * std::mem::size_of::<GpuElementForces>()) as wgpu::BufferAddress;
        let elem_forces_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Element Forces"), size: elem_forces_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Staging buffers for CPU readback
        let elements_size =
            (n_elements * std::mem::size_of::<GpuElement>()) as wgpu::BufferAddress;
        let elements_staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Elements Staging"), size: elements_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let elem_forces_staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Element Forces Staging"), size: elem_forces_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // ── Bind groups ───────────────────────────────────────────────────────
        let update_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Update Elements BG"), layout: &self.update_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: nodes_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: elements_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: material_buf.as_entire_binding() },
            ],
        });

        let force_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Force Computation BG"), layout: &self.force_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: nodes_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: elements_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: elem_forces_buf.as_entire_binding() },
            ],
        });

        // ── Encode both passes in one submit ──────────────────────────────────
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("CPD Two-Pass Encoder"),
        });

        // Pass 1 — strain → stress → is_broken
        {
            let mut cp = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Update Elements"), timestamp_writes: None,
            });
            cp.set_pipeline(&self.update_pipeline);
            cp.set_bind_group(0, &update_bg, &[]);
            cp.dispatch_workgroups(((n_elements as u32) + 63) / 64, 1, 1);
        }

        // Pass 2 — force computation (per-element, no atomics)
        {
            let mut cp = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Force Computation"), timestamp_writes: None,
            });
            cp.set_pipeline(&self.force_pipeline);
            cp.set_bind_group(0, &force_bg, &[]);
            cp.dispatch_workgroups(((n_elements as u32) + 63) / 64, 1, 1);
        }

        if read_elements {
            encoder.copy_buffer_to_buffer(&elements_buf, 0, &elements_staging, 0, elements_size);
        }
        encoder.copy_buffer_to_buffer(&elem_forces_buf,  0, &elem_forces_staging,  0, elem_forces_size);
        self.queue.submit(Some(encoder.finish()));

        // ── Async readback ────────────────────────────────────────────────────
        let rx_e = if read_elements {
            let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
            elements_staging.slice(..).map_async(wgpu::MapMode::Read, move |v| tx.send(v).unwrap());
            Some(rx)
        } else {
            None
        };

        let (tx_f, rx_f) = futures_intrusive::channel::shared::oneshot_channel();
        elem_forces_staging.slice(..).map_async(wgpu::MapMode::Read, move |v| tx_f.send(v).unwrap());

        self.device.poll(wgpu::Maintain::Wait);
        pollster::block_on(async {
            if let Some(rx) = rx_e {
                rx.receive().await.unwrap().unwrap();
            }
            rx_f.receive().await.unwrap().unwrap();
        });

        // ── Decode results ────────────────────────────────────────────────────
        let element_results: Vec<GpuElementResult> = if read_elements {
            let mapped = elements_staging.slice(..).get_mapped_range();
            let raw: &[GpuElement] = bytemuck::cast_slice(&mapped);
            let res = raw.iter().map(|el| GpuElementResult {
                stress: [
                    [el.stress_col0[0], el.stress_col0[1], el.stress_col0[2]],
                    [el.stress_col1[0], el.stress_col1[1], el.stress_col1[2]],
                    [el.stress_col2[0], el.stress_col2[1], el.stress_col2[2]],
                ],
                strain_energy: f32::from_bits(el.strain_energy_bits),
                is_broken:     el.is_broken != 0,
                is_inverted:   el.is_inverted != 0,
            }).collect();
            drop(mapped);
            elements_staging.unmap();
            res
        } else {
            vec![]
        };

        let forces: Vec<[f32; 3]> = {
            use rayon::prelude::*;
            use std::sync::atomic::{AtomicU32, Ordering};

            struct AtomicF32(AtomicU32);
            impl AtomicF32 {
                fn new() -> Self { Self(AtomicU32::new(0f32.to_bits())) }
                fn add(&self, v: f32) {
                    let mut current = self.0.load(Ordering::Relaxed);
                    loop {
                        let new_val = f32::from_bits(current) + v;
                        match self.0.compare_exchange_weak(current, new_val.to_bits(), Ordering::Relaxed, Ordering::Relaxed) {
                            Ok(_) => break,
                            Err(e) => current = e,
                        }
                    }
                }
                fn get(&self) -> f32 { f32::from_bits(self.0.load(Ordering::Relaxed)) }
            }

            let mapped = elem_forces_staging.slice(..).get_mapped_range();
            let raw: &[GpuElementForces] = bytemuck::cast_slice(&mapped);
            
            let node_forces_atomic: Vec<[AtomicF32; 3]> = (0..num_nodes)
                .map(|_| [AtomicF32::new(), AtomicF32::new(), AtomicF32::new()])
                .collect();

            raw.par_iter().enumerate().for_each(|(el_idx, ef)| {
                let indices = gpu_elements[el_idx].node_indices;
                let add_atomic = |dst: &[AtomicF32; 3], src: &[f32; 4]| {
                    dst[0].add(src[0]);
                    dst[1].add(src[1]);
                    dst[2].add(src[2]);
                };
                add_atomic(&node_forces_atomic[indices[0] as usize], &ef.force_a);
                add_atomic(&node_forces_atomic[indices[1] as usize], &ef.force_b);
                add_atomic(&node_forces_atomic[indices[2] as usize], &ef.force_c);
                add_atomic(&node_forces_atomic[indices[3] as usize], &ef.force_d);
            });

            drop(mapped);
            elem_forces_staging.unmap();

            node_forces_atomic.into_iter()
                .map(|f| [f[0].get(), f[1].get(), f[2].get()])
                .collect()
        };

        (forces, element_results)
    }
}

impl GpuSession {
    pub fn execute(
        &mut self,
        gpu_nodes:    &[GpuNode],
        gpu_elements: &[GpuElement],
        material:     &GpuMaterial,
        read_elements: bool,
    ) -> (Vec<[f32; 3]>, Vec<GpuElementResult>) {
        let device = &self.pipeline.device;
        let queue = &self.pipeline.queue;

        // ── Update persistent buffers ─────────────────────────────────────────
        queue.write_buffer(&self.nodes_buf, 0, bytemuck::cast_slice(gpu_nodes));
        queue.write_buffer(&self.elements_buf, 0, bytemuck::cast_slice(gpu_elements));
        queue.write_buffer(&self.material_buf, 0, bytemuck::bytes_of(material));

        // ── Encode passes ─────────────────────────────────────────────────────
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("GpuSession Encoder"),
        });

        {
            let mut cp = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Pass 1: Update Elements"), timestamp_writes: None,
            });
            cp.set_pipeline(&self.pipeline.update_pipeline);
            cp.set_bind_group(0, &self.update_bg, &[]);
            cp.dispatch_workgroups(((self.num_elements as u32) + 63) / 64, 1, 1);
        }

        {
            let mut cp = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Pass 2: Force Computation"), timestamp_writes: None,
            });
            cp.set_pipeline(&self.pipeline.force_pipeline);
            cp.set_bind_group(0, &self.force_bg, &[]);
            cp.dispatch_workgroups(((self.num_elements as u32) + 63) / 64, 1, 1);
        }

        // Copy to staging
        let elements_size = (self.num_elements * std::mem::size_of::<GpuElement>()) as wgpu::BufferAddress;
        let elem_forces_size = (self.num_elements * std::mem::size_of::<GpuElementForces>()) as wgpu::BufferAddress;
        
        if read_elements {
            encoder.copy_buffer_to_buffer(&self.elements_buf, 0, &self.elements_staging, 0, elements_size);
        }
        encoder.copy_buffer_to_buffer(&self.elem_forces_buf, 0, &self.elem_forces_staging, 0, elem_forces_size);

        queue.submit(Some(encoder.finish()));

        // ── Async readback ────────────────────────────────────────────────────
        let rx_e = if read_elements {
            let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
            self.elements_staging.slice(..).map_async(wgpu::MapMode::Read, move |v| tx.send(v).unwrap());
            Some(rx)
        } else {
            None
        };

        let (tx_f, rx_f) = futures_intrusive::channel::shared::oneshot_channel();
        self.elem_forces_staging.slice(..).map_async(wgpu::MapMode::Read, move |v| tx_f.send(v).unwrap());

        device.poll(wgpu::Maintain::Wait);
        pollster::block_on(async {
            if let Some(rx) = rx_e {
                rx.receive().await.unwrap().unwrap();
            }
            rx_f.receive().await.unwrap().unwrap();
        });

        // ── Decode results ────────────────────────────────────────────────────
        let element_results: Vec<GpuElementResult> = if read_elements {
            let mapped = self.elements_staging.slice(..).get_mapped_range();
            let raw: &[GpuElement] = bytemuck::cast_slice(&mapped);
            let res = raw.iter().map(|el| GpuElementResult {
                stress: [
                    [el.stress_col0[0], el.stress_col0[1], el.stress_col0[2]],
                    [el.stress_col1[0], el.stress_col1[1], el.stress_col1[2]],
                    [el.stress_col2[0], el.stress_col2[1], el.stress_col2[2]],
                ],
                strain_energy: f32::from_bits(el.strain_energy_bits),
                is_broken:     el.is_broken != 0,
                is_inverted:   el.is_inverted != 0,
            }).collect();
            drop(mapped);
            self.elements_staging.unmap();
            res
        } else {
            vec![]
        };

        let forces: Vec<[f32; 3]> = {
            use rayon::prelude::*;
            use std::sync::atomic::{AtomicU32, Ordering};

            struct AtomicF32(AtomicU32);
            impl AtomicF32 {
                fn new() -> Self { Self(AtomicU32::new(0f32.to_bits())) }
                fn add(&self, v: f32) {
                    let mut current = self.0.load(Ordering::Relaxed);
                    loop {
                        let new_val = f32::from_bits(current) + v;
                        match self.0.compare_exchange_weak(current, new_val.to_bits(), Ordering::Relaxed, Ordering::Relaxed) {
                            Ok(_) => break,
                            Err(e) => current = e,
                        }
                    }
                }
                fn get(&self) -> f32 { f32::from_bits(self.0.load(Ordering::Relaxed)) }
            }

            let mapped = self.elem_forces_staging.slice(..).get_mapped_range();
            let raw: &[GpuElementForces] = bytemuck::cast_slice(&mapped);
            
            let node_forces_atomic: Vec<[AtomicF32; 3]> = (0..self.num_nodes)
                .map(|_| [AtomicF32::new(), AtomicF32::new(), AtomicF32::new()])
                .collect();

            raw.par_iter().enumerate().for_each(|(el_idx, ef)| {
                let indices = gpu_elements[el_idx].node_indices;
                let add_atomic = |dst: &[AtomicF32; 3], src: &[f32; 4]| {
                    dst[0].add(src[0]);
                    dst[1].add(src[1]);
                    dst[2].add(src[2]);
                };
                add_atomic(&node_forces_atomic[indices[0] as usize], &ef.force_a);
                add_atomic(&node_forces_atomic[indices[1] as usize], &ef.force_b);
                add_atomic(&node_forces_atomic[indices[2] as usize], &ef.force_c);
                add_atomic(&node_forces_atomic[indices[3] as usize], &ef.force_d);
            });

            drop(mapped);
            self.elem_forces_staging.unmap();

            node_forces_atomic.into_iter()
                .map(|f| [f[0].get(), f[1].get(), f[2].get()])
                .collect()
        };

        (forces, element_results)
    }
}
