use super::{
    boundary_average::ForceAndDisplacement, boundary_condition::BoundaryCondition, config::Config,
    element::Element, node, BoundaryAverage, BoundaryInfo, ExportData, Matrix3, Node,
    TimeSeriesValue, TimeStampedValue,
};
use cgal::{triangulation, BoundaryId};
use fxhash::{FxHashMap, FxHashSet};
use nalgebra::Vector3;
use rayon::prelude::*;
use std::{
    cmp::Ordering,
    fmt::Debug,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub struct Unconfigured;

#[derive(Debug)]
pub struct InProgress {
    steps: u128,
    iterations: u128,
    runtime: Option<Duration>,
    config: Box<Config>,
    #[cfg(feature = "gpu")]
    pub(crate) gpu_pipeline: Option<std::sync::Arc<cpd_wgpu::ComputePipeline>>,
    #[cfg(feature = "gpu")]
    pub(crate) gpu_session: Option<cpd_wgpu::GpuSession>,
    pub(crate) h_min: f32,
    pub(crate) stable_steps: u32,
}

#[derive(Debug)]
pub struct Done {
    steps: u128,
    iterations: u128,
    runtime: Option<Duration>,
    config: Box<Config>,
    #[cfg(feature = "gpu")]
    pub(crate) gpu_pipeline: Option<std::sync::Arc<cpd_wgpu::ComputePipeline>>,
    #[cfg(feature = "gpu")]
    pub(crate) gpu_session: Option<cpd_wgpu::GpuSession>,
    pub(crate) h_min: f32,
    pub(crate) stable_steps: u32,
}

#[sealed::sealed]
pub trait State {
    fn iterations(&self) -> u128;
    fn runtime(&self) -> Option<Duration>;
    fn time_elapsed(&self) -> f32;
    fn config(&self) -> Option<&Config>;
}

#[sealed::sealed]
impl State for Unconfigured {
    fn iterations(&self) -> u128 {
        0
    }

    fn runtime(&self) -> Option<Duration> {
        Some(Duration::ZERO)
    }

    fn time_elapsed(&self) -> f32 {
        0.0
    }

    fn config(&self) -> Option<&Config> {
        None
    }
}

#[sealed::sealed]
impl State for InProgress {
    fn iterations(&self) -> u128 {
        self.iterations
    }

    fn runtime(&self) -> Option<Duration> {
        self.runtime
    }

    fn time_elapsed(&self) -> f32 {
        self.config.time_delta().as_secs_f32() * self.iterations as f32
    }

    fn config(&self) -> Option<&Config> {
        Some(&self.config)
    }
}

#[sealed::sealed]
impl State for Done {
    fn iterations(&self) -> u128 {
        self.iterations
    }

    fn runtime(&self) -> Option<Duration> {
        self.runtime
    }

    fn time_elapsed(&self) -> f32 {
        self.config.duration().as_secs_f32()
    }

    fn config(&self) -> Option<&Config> {
        Some(&self.config)
    }
}

#[derive(Debug)]
pub struct Computer<S: State> {
    nodes: Box<[Node]>,
    elements: Box<[Element]>,
    state: S,
    boundary_infos: FxHashMap<BoundaryId, BoundaryInfo>,
    data_recorded_boundary: Option<(FxHashSet<usize>, BoundaryAverage)>,
}

impl<S: State> Computer<S> {
    fn min_max_stress(&self) -> (Matrix3<f32>, Matrix3<f32>) {
        let iter = || self.elements.iter().map(Element::stress).copied();
        iter().zip(iter()).par_bridge().reduce(
            || (Matrix3::from_element(f32::MAX), Matrix3::zeros()),
            |a, b| (a.0.zip_map(&b.0, f32::min), a.1.zip_map(&b.1, f32::max)),
        )
    }

    pub fn iterations(&self) -> u128 {
        self.state.iterations()
    }

    pub fn runtime(&self) -> Option<Duration> {
        self.state.runtime()
    }

    pub fn record_stress_data(&mut self, index: usize) {
        let time_stamp = self.state.time_elapsed();
        let element = &mut self.elements[index];
        element.stress_time_series = TimeSeriesValue::Series(vec![TimeStampedValue {
            time_stamp,
            value: *element.stress(),
        }]);
    }

    pub fn stop_recording_stress_data(&mut self, index: usize) {
        let element = &mut self.elements[index];
        element.stress_time_series = TimeSeriesValue::Single(*element.stress());
    }

    pub fn record_vertex_position(&mut self, index: usize) {
        let time_stamp = self.state.time_elapsed();
        let node = &mut self.nodes[index];
        node.position_time_series = TimeSeriesValue::Series(vec![TimeStampedValue {
            time_stamp,
            value: *node.position(),
        }]);
    }

    pub fn stop_recording_vertex_position(&mut self, index: usize) {
        self.nodes[index].position_time_series =
            TimeSeriesValue::Single(*self.nodes[index].position());
    }

    pub fn record_boundary_data(&mut self, id: BoundaryId) {
        let info = &self.boundary_infos[&id];
        let data = match info.boundary_condition {
            BoundaryCondition::Free => BoundaryAverage::ForceAndDisplacement(vec![]),
            BoundaryCondition::Force(_) => BoundaryAverage::Displacement(vec![]),
            BoundaryCondition::Displacement(_) => BoundaryAverage::Force(vec![]),
        };
        self.data_recorded_boundary = Some((info.node_indices.clone(), data));
    }

    pub fn stop_recording_boundary_data(&mut self) {
        self.data_recorded_boundary = None;
    }

    fn reset_boundary_data(&mut self) {
        if let Some((_, data)) = &mut self.data_recorded_boundary {
            data.reset();
        }
    }

    pub fn export_data(&self) -> ExportData {
        let (min_stress, max_stress) = self.min_max_stress();
        ExportData {
            nodes: self.nodes.clone().into(),
            elements: self.elements.clone().into(),
            boundary_infos: self.boundary_infos.clone(),
            boundary_average_data: self
                .data_recorded_boundary
                .as_ref()
                .map(|(_, data)| data.clone()),
            config: self.state.config().copied(),
            iterations: self.iterations(),
            min_stress,
            max_stress,
        }
    }
}

fn apply_config(
    mut nodes: Box<[Node]>,
    initial_density: f32,
    config: Config,
    reset: bool,
) -> (Box<[Node]>, InProgress) {
    let scale = config.material_props().bulk_props().density() / initial_density;
    nodes.par_iter_mut().for_each(|node| {
        node.scale_mass(scale);
        if reset {
            node.reset();
        }
    });
    let steps = config.duration().as_nanos() / config.time_delta().as_nanos();
    (
        nodes,
        InProgress {
            steps,
            iterations: 0,
            runtime: Some(Duration::ZERO),
            config: Box::new(config),
            #[cfg(feature = "gpu")]
            gpu_pipeline: None,
            #[cfg(feature = "gpu")]
            gpu_session: None,
            h_min: 1e-4, // Will be overridden in configure
            stable_steps: 0,
        },
    )
}

impl Computer<Unconfigured> {
    pub fn configure(self, config: Config) -> Computer<InProgress> {
        let (nodes, mut state) = apply_config(self.nodes, 1.0, config, false);
        
        let h_min = self.elements.par_iter().map(|el| {
            let n0 = nodes[el.indices[0]].initial_position();
            let n1 = nodes[el.indices[1]].initial_position();
            let n2 = nodes[el.indices[2]].initial_position();
            let n3 = nodes[el.indices[3]].initial_position();
            let d01 = (n0 - n1).norm_squared();
            let d02 = (n0 - n2).norm_squared();
            let d03 = (n0 - n3).norm_squared();
            let d12 = (n1 - n2).norm_squared();
            let d13 = (n1 - n3).norm_squared();
            let d23 = (n2 - n3).norm_squared();
            let min_d_sq = [d01, d02, d03, d12, d13, d23].into_iter().reduce(f32::min).unwrap();
            min_d_sq.max(1e-12)
        }).reduce(|| f32::MAX, f32::min).sqrt();
        state.h_min = h_min;

        Computer {
            nodes,
            state,
            elements: self.elements,
            boundary_infos: self.boundary_infos,
            data_recorded_boundary: self.data_recorded_boundary,
        }
    }
}

fn delaunay_deformation_tensor(
    d_ba: &Vector3<f32>,
    d_ca: &Vector3<f32>,
    d_da: &Vector3<f32>,
    r_ba: &Vector3<f32>,
    r_ca: &Vector3<f32>,
    r_da: &Vector3<f32>,
) -> Option<Matrix3<f32>> {
    let d = Matrix3::from_columns(&[*d_ba, *d_ca, *d_da]);
    let r = Matrix3::from_columns(&[*r_ba, *r_ca, *r_da]);
    r.try_inverse().map(|inv| d * inv)
}

fn green_lagrange_strain_tensor(f: &Matrix3<f32>) -> Matrix3<f32> {
    let c: Matrix3<f32> = f.transpose() * f;
    (c - Matrix3::identity()) / 2.0
}

fn strain_energy(stress: &Matrix3<f32>, strain: &Matrix3<f32>) -> f32 {
    stress.component_mul(strain).sum() / 2.0
}

fn update_element(time_stamp: f32, element: &mut Element, config: &Config, nodes: [&Node; 4]) {
    let r_ba: Vector3<f32> = nodes[1].initial_position() - nodes[0].initial_position();
    let r_ca: Vector3<f32> = nodes[2].initial_position() - nodes[0].initial_position();
    let r_da: Vector3<f32> = nodes[3].initial_position() - nodes[0].initial_position();

    let d_ba: Vector3<f32> = nodes[1].position() - nodes[0].position();
    let d_ca: Vector3<f32> = nodes[2].position() - nodes[0].position();
    let d_da: Vector3<f32> = nodes[3].position() - nodes[0].position();

    if let Some(mut f) = delaunay_deformation_tensor(&d_ba, &d_ca, &d_da, &r_ba, &r_ca, &r_da) {
        if f.determinant() <= 0.0 {
            element.is_inverted = true;
            f = Matrix3::identity();
        } else {
            element.is_inverted = false;
        }
        element.strain = green_lagrange_strain_tensor(&f);
        element.stress_time_series.set_or_push(
            time_stamp,
            config.material_props().eval_stress(&element.strain),
        );
        element.strain_energy = strain_energy(element.stress(), &element.strain);
        element.is_broken = config
            .material_props()
            .bulk_props()
            .failure_criteria()
            .satisfies(element.strain_energy, element.stress());
    } else {
        element.is_broken = true;
    }
}


pub enum AdvanceResult {
    InProgress(Computer<InProgress>),
    Done(Computer<Done>),
}

impl Computer<InProgress> {
    pub fn reconfigure(mut self, config: Config) -> Computer<InProgress> {
        let (nodes, state) = apply_config(
            self.nodes,
            *self.state.config.material_props().bulk_props().density(),
            config,
            self.state.iterations > 0,
        );
        self.elements.par_iter_mut().for_each(Element::reset);
        Computer {
            nodes,
            state,
            elements: self.elements,
            boundary_infos: self.boundary_infos,
            data_recorded_boundary: self.data_recorded_boundary,
        }
    }

    pub fn progress(&self) -> f32 {
        (self.iterations() as f32) / (self.total_iterations() as f32)
    }

    pub fn total_iterations(&self) -> u128 {
        self.state.steps
    }

    fn update_boundary_data(&mut self, time_stamp: f32) {
        let Some((node_indices, data)) = &mut self.data_recorded_boundary else {
            return;
        };
        let node_indices = &*node_indices;
        macro_rules! sum {
            ($map_node:expr) => {
                node_indices
                    .par_iter()
                    .map(|index| &self.nodes[*index])
                    .map($map_node)
                    .sum()
            };
        }
        let boundary_nodes = node_indices.len() as f32;
        match data {
            BoundaryAverage::Force(series) => {
                let sum: Vector3<f32> = sum!(|node| node.force());
                series.push(TimeStampedValue {
                    time_stamp,
                    value: sum / boundary_nodes,
                });
            }
            BoundaryAverage::Displacement(series) => {
                let sum: Vector3<f32> = sum!(|node| node.position());
                series.push(TimeStampedValue {
                    time_stamp,
                    value: sum / boundary_nodes,
                });
            }
            BoundaryAverage::ForceAndDisplacement(series) => {
                let mut sum: ForceAndDisplacement = sum!(|node| ForceAndDisplacement {
                    force: node.force,
                    displacement: *node.position()
                });
                sum.force /= boundary_nodes;
                sum.displacement /= boundary_nodes;
                series.push(TimeStampedValue {
                    time_stamp,
                    value: sum,
                });
            }
        }
    }

    pub fn advance(mut self, read_elements: bool) -> AdvanceResult {
        let now = Instant::now();
        
        // --- Adaptive Time Step (CFL Condition) ---
        if *self.state.config.adaptive_time_step() {
            let v_max = self.nodes.par_iter().map(|n| n.velocity().norm()).reduce(|| 0.0, f32::max);
            let h_min = self.state.h_min;
            let mut dt = self.state.config.time_delta().as_secs_f32();
            
            if v_max > 1e-8 && h_min > 1e-8 {
                let courant = v_max * dt / h_min;
                if courant > 0.5 {
                    dt *= 0.5;
                    self.state.stable_steps = 0;
                } else if courant < 0.1 {
                    self.state.stable_steps += 1;
                    if self.state.stable_steps >= 10 {
                        dt = (dt * 1.25).min(self.state.config.max_time_delta().map(|d| d.as_secs_f32()).unwrap_or(1e-2));
                        self.state.stable_steps = 0;
                    }
                } else {
                    self.state.stable_steps = 0;
                }
            }
            if let Some(min_dt) = self.state.config.min_time_delta() {
                dt = dt.max(min_dt.as_secs_f32());
            }
            if let Some(max_dt) = self.state.config.max_time_delta() {
                dt = dt.min(max_dt.as_secs_f32());
            }
            
            self.state.config.set_time_delta(Duration::from_secs_f32(dt));
            
            // Recalculate remaining steps based on new time delta
            // Use f64 for all calculations to avoid overflow and precision loss
            let total_duration = self.state.config.duration().as_secs_f64();
            let time_elapsed = self.state.iterations as f64 * dt as f64; // Approximation if dt changed recently
            let remaining_time = total_duration - time_elapsed;
            
            if remaining_time > 0.0 && dt > 1e-12 {
                let remaining_steps = (remaining_time / dt as f64).ceil() as u128;
                self.state.steps = self.state.iterations + remaining_steps;
            }
        }
        
        let config = &self.state.config;
        let time_stamp = self.state.time_elapsed();
        let iterations = self.state.iterations;
        let damping = *config.material_props().bulk_props().damping();
        let time_delta = config.time_delta().as_secs_f32();
        
        #[cfg(feature = "gpu")]
        let mut session_opt = self.state.gpu_session.take();
        
        #[cfg(feature = "gpu")]
        let forces = if let Some(session) = &mut session_opt {
            let f = self.advance_gpu(session, time_stamp, read_elements);
            self.state.gpu_session = session_opt;
            f
        } else {
            self.advance_cpu(time_stamp)
        };
        
        #[cfg(not(feature = "gpu"))]
        let forces = self.advance_cpu(time_stamp);

        // Position update has to be done at the end otherwise it
        // will interfere with force calculation
        forces
            .into_par_iter()
            .zip(self.nodes.par_iter_mut())
            .for_each(|(force, node)| {
                node.apply_force_and_bc(
                    force,
                    iterations,
                    damping,
                    time_delta,
                )
            });
        
        self.update_boundary_data(time_stamp);
        self.state.iterations += 1;
        if let Some(runtime) = &mut self.state.runtime {
            *runtime += now.elapsed();
        }
        
        let in_progress = self.state;
        if in_progress.iterations >= in_progress.steps {
            AdvanceResult::Done(Computer::<Done> {
                nodes: self.nodes,
                elements: self.elements,
                state: Done {
                    steps: in_progress.steps,
                    iterations: in_progress.iterations,
                    runtime: in_progress.runtime,
                    config: in_progress.config,
                    #[cfg(feature = "gpu")]
                    gpu_pipeline: in_progress.gpu_pipeline,
                    #[cfg(feature = "gpu")]
                    gpu_session: in_progress.gpu_session,
                    h_min: in_progress.h_min,
                    stable_steps: in_progress.stable_steps,
                },
                boundary_infos: self.boundary_infos,
                data_recorded_boundary: self.data_recorded_boundary,
            })
        } else {
            AdvanceResult::InProgress(Computer::<InProgress> {
                nodes: self.nodes,
                elements: self.elements,
                state: in_progress,
                boundary_infos: self.boundary_infos,
                data_recorded_boundary: self.data_recorded_boundary,
            })
        }
    }

    fn advance_cpu(&mut self, time_stamp: f32) -> Vec<Vector3<f32>> {
        let config = &self.state.config;
        self.elements
            .par_iter_mut()
            .filter(|element| !element.is_broken)
            .for_each(|element| {
                let node = |i| &self.nodes[element.indices[i]];
                update_element(time_stamp, element, config, [node(0), node(1), node(2), node(3)])
            });
        
        use std::sync::atomic::{AtomicU32, Ordering};
        struct AtomicF32(AtomicU32);
        impl AtomicF32 {
            fn new() -> Self {
                Self(AtomicU32::new(0f32.to_bits()))
            }
            fn add(&self, v: f32) {
                let mut current = self.0.load(Ordering::Relaxed);
                loop {
                    let new_val = f32::from_bits(current) + v;
                    match self.0.compare_exchange_weak(
                        current, new_val.to_bits(),
                        Ordering::Relaxed, Ordering::Relaxed
                    ) {
                        Ok(_) => break,
                        Err(e) => current = e,
                    }
                }
            }
            fn get(&self) -> f32 {
                f32::from_bits(self.0.load(Ordering::Relaxed))
            }
        }

        // Lock-free parallel accumulation
        let forces: Vec<[AtomicF32; 3]> = (0..self.nodes.len())
            .map(|_| [AtomicF32::new(), AtomicF32::new(), AtomicF32::new()])
            .collect();

        self.elements
            .par_iter()
            .filter(|element| !element.is_broken)
            .for_each(|element| {
                let node = |i| &self.nodes[element.indices[i]];

                let d_ba = node(1).position() - node(0).position();
                let d_ca = node(2).position() - node(0).position();
                let d_da = node(3).position() - node(0).position();
                let d = Matrix3::from_columns(&[d_ba, d_ca, d_da]);

                let r_ba = node(1).initial_position() - node(0).initial_position();
                let r_ca = node(2).initial_position() - node(0).initial_position();
                let r_da = node(3).initial_position() - node(0).initial_position();
                let r = Matrix3::from_columns(&[r_ba, r_ca, r_da]);

                let volume = r.determinant().abs() / 6.0;
                let h = r.try_inverse().unwrap_or_else(Matrix3::zeros);

                let f_mat = d * h;
                let p = f_mat * element.stress();

                let h_t = h.transpose();
                let grad_b = h_t.column(0).into_owned();
                let grad_c = h_t.column(1).into_owned();
                let grad_d = h_t.column(2).into_owned();

                let force_b = -volume * (p * grad_b);
                let force_c = -volume * (p * grad_c);
                let force_d = -volume * (p * grad_d);
                let force_a = -(force_b + force_c + force_d);

                let forces_arr = [force_a, force_b, force_c, force_d];
                for i in 0..4 {
                    let idx = element.indices[i];
                    let f = forces_arr[i];
                    forces[idx][0].add(f.x);
                    forces[idx][1].add(f.y);
                    forces[idx][2].add(f.z);
                }
            });

        forces
            .into_iter()
            .map(|f| Vector3::new(f[0].get(), f[1].get(), f[2].get()))
            .collect()
    }

    #[cfg(feature = "gpu")]
    fn advance_gpu(&mut self, session: &mut cpd_wgpu::GpuSession, time_stamp: f32, read_elements: bool) -> Vec<Vector3<f32>> {
        use cpd_wgpu::{GpuElement, GpuMaterial, GpuNode};

        let config = &self.state.config;

        // ── Build GpuMaterial from the current Config ─────────────────────
        let mat = config.material_props();
        let bp = mat.bulk_props();
        let fc = bp.failure_criteria();

        let gpu_material: GpuMaterial = match mat {
            super::material::Props::Isotropic(p) => GpuMaterial {
                density:               *bp.density(),
                damping:               *bp.damping(),
                failure_strain_energy: (*fc.strain_energy()).unwrap_or(0.0),
                failure_tensile:       (*fc.tensional_stress()).unwrap_or(0.0),
                failure_compressive:   (*fc.compressional_stress()).unwrap_or(0.0),
                material_type:         0,
                elasticity_modulus:    *p.elasticity_modulus(),
                poissons_ratio:        *p.poissons_ratio(),
                ex: 0.0, ey: 0.0, ez: 0.0,
                nu_xy: 0.0, nu_yx: 0.0,
                nu_yz: 0.0, nu_zy: 0.0,
                nu_zx: 0.0, nu_xz: 0.0,
                g_xy: 0.0, g_yz: 0.0, g_zx: 0.0,
                _pad: [0.0; 3],
            },
            super::material::Props::Orthotropic(p) => GpuMaterial {
                density:               *bp.density(),
                damping:               *bp.damping(),
                failure_strain_energy: (*fc.strain_energy()).unwrap_or(0.0),
                failure_tensile:       (*fc.tensional_stress()).unwrap_or(0.0),
                failure_compressive:   (*fc.compressional_stress()).unwrap_or(0.0),
                material_type:         1,
                elasticity_modulus:    0.0,
                poissons_ratio:        0.0,
                ex:    *p.elasticity_modulus_x(),
                ey:    *p.elasticity_modulus_y(),
                ez:    *p.elasticity_modulus_z(),
                nu_xy: *p.poissons_ratio_xy(),
                nu_yx: *p.poissons_ratio_yx(),
                nu_yz: *p.poissons_ratio_yz(),
                nu_zy: *p.poissons_ratio_zy(),
                nu_zx: *p.poissons_ratio_zx(),
                nu_xz: *p.poissons_ratio_xz(),
                g_xy:  *p.shear_modulus_xy(),
                g_yz:  *p.shear_modulus_yz(),
                g_zx:  *p.shear_modulus_zx(),
                _pad: [0.0; 3],
            },
        };

        // ── Upload node and element data ──────────────────────────────────
        let gpu_nodes: Vec<GpuNode> = self.nodes.par_iter().map(|node| {
            let p  = node.position();
            let ip = node.initial_position();
            let v  = node.velocity();
            GpuNode {
                initial_position: [ip.x, ip.y, ip.z, 0.0],
                position:         [p.x,  p.y,  p.z,  0.0],
                velocity:         [v.x,  v.y,  v.z,  0.0],
                mass:             node.mass(),
                _padding:         [0.0; 3],
            }
        }).collect();

        // Upload element topology + previous stress as seed (Pass 1 will overwrite)
        let gpu_elements: Vec<GpuElement> = self.elements.par_iter().map(|el| {
            let s = el.stress();
            GpuElement {
                node_indices:       [el.indices[0] as u32, el.indices[1] as u32,
                                     el.indices[2] as u32, el.indices[3] as u32],
                stress_col0:        [s[(0,0)], s[(1,0)], s[(2,0)], 0.0],
                stress_col1:        [s[(0,1)], s[(1,1)], s[(2,1)], 0.0],
                stress_col2:        [s[(0,2)], s[(1,2)], s[(2,2)], 0.0],
                is_broken:          if el.is_broken { 1 } else { 0 },
                strain_energy_bits: el.strain_energy.to_bits(),
                is_inverted:        if el.is_inverted { 1 } else { 0 },
                _padding:           0,
            }
        }).collect();

        // ── Run both GPU passes ───────────────────────────────────────────
        let (gpu_forces, element_results) =
            session.execute(&gpu_nodes, &gpu_elements, &gpu_material, read_elements);

        // ── Write GPU element results back to CPU state ───────────────────
        // This preserves time-series recording and export data continuity
        // without any redundant CPU physics computation.
        self.elements
            .iter_mut()
            .zip(element_results.iter())
            .for_each(|(el, res)| {
                // Only update non-broken elements (a broken element stays broken)
                if !el.is_broken {
                    // Reconstruct nalgebra Matrix3 from column arrays
                    let stress = Matrix3::from_columns(&[
                        Vector3::new(res.stress[0][0], res.stress[0][1], res.stress[0][2]),
                        Vector3::new(res.stress[1][0], res.stress[1][1], res.stress[1][2]),
                        Vector3::new(res.stress[2][0], res.stress[2][1], res.stress[2][2]),
                    ]);
                    el.stress_time_series.set_or_push(time_stamp, stress);
                    el.strain_energy = res.strain_energy;
                    el.is_broken     = res.is_broken;
                    el.is_inverted   = res.is_inverted;
                }
            });

        // ── Convert GPU forces to nalgebra vectors ────────────────────────
        gpu_forces
            .into_iter()
            .map(|f| Vector3::new(f[0], f[1], f[2]))
            .collect()
    }


    #[cfg(feature = "gpu")]
    pub fn set_gpu_pipeline(&mut self, pipeline: Option<std::sync::Arc<cpd_wgpu::ComputePipeline>>) {
        self.state.gpu_pipeline = pipeline.clone();
        if let Some(p) = pipeline {
            self.state.gpu_session = Some(p.create_session(self.nodes.len(), self.elements.len()));
        } else {
            self.state.gpu_session = None;
        }
    }

    pub fn set_duration(&mut self, duration: Duration) -> Result<(), String> {
        let config = &mut self.state.config;
        let completed_duration = (self.state.iterations as f32) * config.time_delta().as_secs_f32();
        let completed_duration = Duration::from_secs_f32(completed_duration);
        if duration < completed_duration {
            Err(String::from(
                "Cannot reduce duration to a value which is less than duration elapsed",
            ))
        } else {
            config.set_duration(duration);
            self.state.steps = config.duration().as_nanos() / config.time_delta().as_nanos();
            Ok(())
        }
    }

    pub fn reset(&mut self) {
        self.nodes.par_iter_mut().for_each(Node::reset);
        self.elements.par_iter_mut().for_each(Element::reset);
        self.state.iterations = 0;
        self.state.runtime = Some(Duration::ZERO);
        self.reset_boundary_data();
    }
}

pub type SetDurationResult = Result<Computer<InProgress>, (Computer<Done>, String)>;

impl Computer<Done> {
    pub fn reconfigure(mut self, config: Config) -> Computer<InProgress> {
        let (nodes, state) = apply_config(
            self.nodes,
            *self.state.config.material_props().bulk_props().density(),
            config,
            true,
        );
        self.elements.par_iter_mut().for_each(Element::reset);
        Computer {
            nodes,
            state,
            elements: self.elements,
            boundary_infos: self.boundary_infos,
            data_recorded_boundary: self.data_recorded_boundary,
        }
    }

    pub fn total_iterations(&self) -> u128 {
        self.state.steps
    }

    pub fn set_duration(mut self, duration: Duration) -> SetDurationResult {
        let completed_duration = (self.state.iterations as f32) * self.state.config.time_delta().as_secs_f32();
        let completed_duration = Duration::from_secs_f32(completed_duration);
        if duration < completed_duration {
            Err((
                self,
                String::from(
                    "Cannot reduce duration to a value which is less than duration elapsed",
                ),
            ))
        } else {
            self.state.config.set_duration(duration);
            let steps = self.state.config.duration().as_nanos() / self.state.config.time_delta().as_nanos();
            Ok(Computer {
                nodes: self.nodes,
                elements: self.elements,
                state: InProgress {
                    steps,
                    iterations: self.state.iterations,
                    runtime: self.state.runtime,
                    config: self.state.config,
                    #[cfg(feature = "gpu")]
                    gpu_pipeline: self.state.gpu_pipeline.clone(),
                    #[cfg(feature = "gpu")]
                    gpu_session: self.state.gpu_session,
                    h_min: self.state.h_min,
                    stable_steps: self.state.stable_steps,
                },
                boundary_infos: self.boundary_infos,
                data_recorded_boundary: self.data_recorded_boundary,
            })
        }
    }

    pub fn reset(mut self) -> Computer<InProgress> {
        let n_nodes = self.nodes.len();
        let n_elements = self.elements.len();
        self.nodes.par_iter_mut().for_each(Node::reset);
        self.elements.par_iter_mut().for_each(Element::reset);
        self.reset_boundary_data();
        Computer {
            nodes: self.nodes,
            elements: self.elements,
            state: InProgress {
                steps: self.state.steps,
                iterations: 0,
                runtime: Some(Duration::ZERO),
                config: self.state.config,
                #[cfg(feature = "gpu")]
                gpu_pipeline: self.state.gpu_pipeline.clone(),
                #[cfg(feature = "gpu")]
                gpu_session: self.state.gpu_session.as_ref().map(|s| {
                    // This is a bit expensive but reset() is rare. 
                    // Better would be to have GpuSession::reset()
                    s.pipeline.clone().create_session(n_nodes, n_elements)
                }),
                h_min: self.state.h_min,
                stable_steps: 0,
            },
            boundary_infos: self.boundary_infos,
            data_recorded_boundary: self.data_recorded_boundary,
        }
    }
}

pub fn unconfigured(
    triangulation_data: &triangulation::Data,
    boundary_point_map: &FxHashMap<BoundaryId, FxHashSet<usize>>,
    boundary_conditions: &FxHashMap<BoundaryId, BoundaryCondition>,
    point_boundary_conditions: &FxHashMap<usize, BoundaryCondition>,
) -> Computer<Unconfigured> {
    let nodes = node::nodes(triangulation_data, point_boundary_conditions, 1.0);
    let elements: Box<[Element]> = triangulation_data
        .faces()
        .par_iter()
        .map(|face| {
            let mut el = Element::new(face.0);
            let n0 = nodes[el.indices[0]].initial_position();
            let n1 = nodes[el.indices[1]].initial_position();
            let n2 = nodes[el.indices[2]].initial_position();
            let n3 = nodes[el.indices[3]].initial_position();
            let r = nalgebra::Matrix3::from_columns(&[n1 - n0, n2 - n0, n3 - n0]);
            if r.determinant().abs() / 6.0 < 1e-6 {
                el.is_broken = true;
            }
            el
        })
        .collect();

    Computer::<Unconfigured> {
        nodes,
        elements,
        state: Unconfigured,
        boundary_infos: boundary_point_map
            .iter()
            .map(|(id, indices)| {
                let info = BoundaryInfo {
                    boundary_condition: boundary_conditions.get(id).cloned().unwrap_or(BoundaryCondition::Free),
                    node_indices: indices.clone(),
                };
                (*id, info)
            })
            .collect(),
        data_recorded_boundary: None,
    }
}

pub enum ImportResult {
    Unconfigured(Computer<Unconfigured>),
    InProgress((Computer<InProgress>, Config)),
    Done((Computer<Done>, Config)),
    Err(String),
}

pub fn from_export_data(export_data: ExportData) -> ImportResult {
    let Some(config) = export_data.config else {
        return ImportResult::Unconfigured(Computer {
                nodes: (*export_data.nodes).to_vec().into_boxed_slice(),
                elements: (*export_data.elements).to_vec().into_boxed_slice(),
            state: Unconfigured,
            boundary_infos: export_data.boundary_infos,
            data_recorded_boundary: None,
        });
    };
    let steps = config.duration().as_nanos() / config.time_delta().as_nanos();
    match export_data.iterations.cmp(&steps) {
        Ordering::Less => ImportResult::InProgress((
            Computer {
                nodes: (*export_data.nodes).to_vec().into_boxed_slice(),
                elements: (*export_data.elements).to_vec().into_boxed_slice(),
                state: InProgress {
                    steps,
                    iterations: export_data.iterations,
                    runtime: (export_data.iterations == 0).then_some(Duration::ZERO),
                    config: Box::new(config),
                    #[cfg(feature = "gpu")]
                    gpu_pipeline: None,
                    #[cfg(feature = "gpu")]
                    gpu_session: None,
                    h_min: 1e-4,
                    stable_steps: 0,
                },
                boundary_infos: export_data.boundary_infos,
                data_recorded_boundary: None,
            },
            config,
        )),
        Ordering::Equal => ImportResult::Done((
            Computer {
                nodes: (*export_data.nodes).to_vec().into_boxed_slice(),
                elements: (*export_data.elements).to_vec().into_boxed_slice(),
                state: Done {
                    steps,
                    iterations: steps,
                    runtime: None,
                    config: Box::new(config),
                    #[cfg(feature = "gpu")]
                    gpu_pipeline: None,
                    #[cfg(feature = "gpu")]
                    gpu_session: None,
                    h_min: 1e-4,
                    stable_steps: 0,
                },
                boundary_infos: export_data.boundary_infos,
                data_recorded_boundary: None,
            },
            config,
        )),
        Ordering::Greater => ImportResult::Err(String::from(
            "Exported data has more iterations than allowed",
        )),
    }
}
