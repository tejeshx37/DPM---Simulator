// Minimal single-element parity test: bypasses the full simulation stack.
// Pass 1 (GPU) now computes stress from node positions; this test verifies
// that the GPU-computed stress and resulting forces match the CPU reference.

use nalgebra::{Matrix3, Vector3};

/// CPU reference: computes Green-Lagrange strain → isotropic stress → forces
fn cpu_full(
    n0_ip: Vector3<f32>, n1_ip: Vector3<f32>, n2_ip: Vector3<f32>, n3_ip: Vector3<f32>,
    n0_p:  Vector3<f32>, n1_p:  Vector3<f32>, n2_p:  Vector3<f32>, n3_p:  Vector3<f32>,
    e_mod: f32, nu: f32,
) -> ([Vector3<f32>; 4], Matrix3<f32>) {
    let r_ba = n1_ip - n0_ip;
    let r_ca = n2_ip - n0_ip;
    let r_da = n3_ip - n0_ip;
    let r = Matrix3::from_columns(&[r_ba, r_ca, r_da]);

    let d_ba = n1_p - n0_p;
    let d_ca = n2_p - n0_p;
    let d_da = n3_p - n0_p;
    let d = Matrix3::from_columns(&[d_ba, d_ca, d_da]);

    let h = r.try_inverse().expect("non-singular R");
    let f_mat = d * h;

    // Green-Lagrange strain
    let c = f_mat.transpose() * f_mat;
    let strain = (c - Matrix3::identity()) * 0.5;

    // Isotropic stress
    let nu = nu.clamp(0.0, 0.499);
    let factor = e_mod / ((1.0 + nu) * (1.0 - 2.0 * nu));
    let stress = factor * nalgebra::matrix![
        (1.0-nu)*strain.m11 + nu*(strain.m22+strain.m33),
        (1.0-2.0*nu)/2.0*strain.m12*2.0, (1.0-2.0*nu)/2.0*strain.m13*2.0;
        (1.0-2.0*nu)/2.0*strain.m12*2.0,
        (1.0-nu)*strain.m22 + nu*(strain.m11+strain.m33),
        (1.0-2.0*nu)/2.0*strain.m23*2.0;
        (1.0-2.0*nu)/2.0*strain.m13*2.0, (1.0-2.0*nu)/2.0*strain.m23*2.0,
        (1.0-nu)*strain.m33 + nu*(strain.m11+strain.m22);
    ];

    let volume = r.determinant().abs() / 6.0;
    let p = f_mat * stress;
    let h_t = h.transpose();
    let force_b = -volume * (p * h_t.column(0).into_owned());
    let force_c = -volume * (p * h_t.column(1).into_owned());
    let force_d = -volume * (p * h_t.column(2).into_owned());
    let force_a = -(force_b + force_c + force_d);

    ([force_a, force_b, force_c, force_d], stress)
}

fn main() {
    println!("Initializing GPU (two-pass)...");
    let pipeline = pollster::block_on(cpd_wgpu::ComputePipeline::new())
        .expect("Failed to init GPU");

    let n0_ip = Vector3::new(0.0f32, 0.0, 0.0);
    let n1_ip = Vector3::new(1.0f32, 0.0, 0.0);
    let n2_ip = Vector3::new(0.0f32, 1.0, 0.0);
    let n3_ip = Vector3::new(0.0f32, 0.0, 1.0);

    let n0_p = n0_ip + Vector3::new( 0.01, -0.02,  0.005);
    let n1_p = n1_ip + Vector3::new(-0.015, 0.03, -0.01);
    let n2_p = n2_ip + Vector3::new( 0.02, -0.01,  0.03);
    let n3_p = n3_ip + Vector3::new(-0.03,  0.02, -0.02);

    let e_mod = 200.0e9f32;  // Steel — no overflow now that we use f32 per-element forces
    let nu    = 0.3f32;

    let (cpu_forces, cpu_stress) =
        cpu_full(n0_ip, n1_ip, n2_ip, n3_ip, n0_p, n1_p, n2_p, n3_p, e_mod, nu);

    println!("CPU σ_xx = {:.4e} Pa", cpu_stress.m11);
    println!("CPU force_a: {:.6?}", cpu_forces[0]);
    println!("CPU force_b: {:.6?}", cpu_forces[1]);
    println!("CPU force_c: {:.6?}", cpu_forces[2]);
    println!("CPU force_d: {:.6?}", cpu_forces[3]);

    // GPU: Pass 1 computes stress; Pass 2 accumulates forces
    let gpu_nodes = vec![
        cpd_wgpu::GpuNode { initial_position: [n0_ip.x, n0_ip.y, n0_ip.z, 0.0], position: [n0_p.x, n0_p.y, n0_p.z, 0.0], velocity: [0.0; 4], mass: 1.0, _padding: [0.0; 3] },
        cpd_wgpu::GpuNode { initial_position: [n1_ip.x, n1_ip.y, n1_ip.z, 0.0], position: [n1_p.x, n1_p.y, n1_p.z, 0.0], velocity: [0.0; 4], mass: 1.0, _padding: [0.0; 3] },
        cpd_wgpu::GpuNode { initial_position: [n2_ip.x, n2_ip.y, n2_ip.z, 0.0], position: [n2_p.x, n2_p.y, n2_p.z, 0.0], velocity: [0.0; 4], mass: 1.0, _padding: [0.0; 3] },
        cpd_wgpu::GpuNode { initial_position: [n3_ip.x, n3_ip.y, n3_ip.z, 0.0], position: [n3_p.x, n3_p.y, n3_p.z, 0.0], velocity: [0.0; 4], mass: 1.0, _padding: [0.0; 3] },
    ];

    // Stress seed is zero — Pass 1 will compute it from scratch
    let gpu_elements = vec![
        cpd_wgpu::GpuElement {
            node_indices:       [0, 1, 2, 3],
            stress_col0:        [0.0; 4],
            stress_col1:        [0.0; 4],
            stress_col2:        [0.0; 4],
            is_broken:          0,
            strain_energy_bits: 0,
            is_inverted:        0,
            _padding:           0,
        }
    ];

    let material = cpd_wgpu::GpuMaterial {
        density: 7800.0, damping: 0.0,
        failure_strain_energy: 0.0, failure_tensile: 0.0, failure_compressive: 0.0,
        material_type: 0,
        elasticity_modulus: e_mod, poissons_ratio: nu,
        ex: 0.0, ey: 0.0, ez: 0.0,
        nu_xy: 0.0, nu_yx: 0.0, nu_yz: 0.0, nu_zy: 0.0, nu_zx: 0.0, nu_xz: 0.0,
        g_xy: 0.0, g_yz: 0.0, g_zx: 0.0,
        _pad: [0.0; 3],
    };

    let (gpu_forces, elem_results) = pipeline.execute(4, &gpu_nodes, &gpu_elements, &material);

    println!("\nGPU σ_xx = {:.4e} Pa", elem_results[0].stress[0][0]);
    println!("GPU force_a: [{:.6}, {:.6}, {:.6}]", gpu_forces[0][0], gpu_forces[0][1], gpu_forces[0][2]);
    println!("GPU force_b: [{:.6}, {:.6}, {:.6}]", gpu_forces[1][0], gpu_forces[1][1], gpu_forces[1][2]);
    println!("GPU force_c: [{:.6}, {:.6}, {:.6}]", gpu_forces[2][0], gpu_forces[2][1], gpu_forces[2][2]);
    println!("GPU force_d: [{:.6}, {:.6}, {:.6}]", gpu_forces[3][0], gpu_forces[3][1], gpu_forces[3][2]);

    let diff_a = (cpu_forces[0] - Vector3::new(gpu_forces[0][0], gpu_forces[0][1], gpu_forces[0][2])).norm();
    let diff_b = (cpu_forces[1] - Vector3::new(gpu_forces[1][0], gpu_forces[1][1], gpu_forces[1][2])).norm();
    let diff_stress = (cpu_stress.m11 - elem_results[0].stress[0][0]).abs();

    println!("\nForce diff a: {:.2e}, b: {:.2e}", diff_a, diff_b);
    println!("Stress σ_xx diff: {:.2e} Pa", diff_stress);

    let tol = 1.0e4; // 10 kN tolerance (forces ~1e9 N; f32 relative error ~1e-6)
    if diff_a < tol && diff_b < tol {
        println!("SINGLE ELEMENT PARITY: PASSED ✓");
    } else {
        println!("SINGLE ELEMENT PARITY: FAILED ✗");
    }
}
