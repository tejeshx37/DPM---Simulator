use nalgebra::{Matrix3, Vector3};
use cpd_wgpu::*;

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

#[test]
fn test_gpu_cpu_parity() {
    let pipeline = pollster::block_on(ComputePipeline::new());
    if pipeline.is_none() {
        eprintln!("No GPU found, skipping parity test.");
        return;
    }
    let pipeline = std::sync::Arc::new(pipeline.unwrap());

    let n0_ip = Vector3::new(0.0f32, 0.0, 0.0);
    let n1_ip = Vector3::new(1.0f32, 0.0, 0.0);
    let n2_ip = Vector3::new(0.0f32, 1.0, 0.0);
    let n3_ip = Vector3::new(0.0f32, 0.0, 1.0);

    let n0_p = n0_ip + Vector3::new( 0.01, -0.02,  0.005);
    let n1_p = n1_ip + Vector3::new(-0.015, 0.03, -0.01);
    let n2_p = n2_ip + Vector3::new( 0.02, -0.01,  0.03);
    let n3_p = n3_ip + Vector3::new(-0.03,  0.02, -0.02);

    let e_mod = 200.0e9f32;
    let nu    = 0.3f32;

    let (cpu_forces, cpu_stress) =
        cpu_full(n0_ip, n1_ip, n2_ip, n3_ip, n0_p, n1_p, n2_p, n3_p, e_mod, nu);

    let gpu_nodes = vec![
        GpuNode { initial_position: [n0_ip.x, n0_ip.y, n0_ip.z, 0.0], position: [n0_p.x, n0_p.y, n0_p.z, 0.0], velocity: [0.0; 4], mass: 1.0, _padding: [0.0; 3] },
        GpuNode { initial_position: [n1_ip.x, n1_ip.y, n1_ip.z, 0.0], position: [n1_p.x, n1_p.y, n1_p.z, 0.0], velocity: [0.0; 4], mass: 1.0, _padding: [0.0; 3] },
        GpuNode { initial_position: [n2_ip.x, n2_ip.y, n2_ip.z, 0.0], position: [n2_p.x, n2_p.y, n2_p.z, 0.0], velocity: [0.0; 4], mass: 1.0, _padding: [0.0; 3] },
        GpuNode { initial_position: [n3_ip.x, n3_ip.y, n3_ip.z, 0.0], position: [n3_p.x, n3_p.y, n3_p.z, 0.0], velocity: [0.0; 4], mass: 1.0, _padding: [0.0; 3] },
    ];

    let gpu_elements = vec![
        GpuElement {
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

    let gpu_material = GpuMaterial {
        density: 7850.0,
        damping: 0.1,
        failure_strain_energy: 1e10,
        failure_tensile: 1e10,
        failure_compressive: 1e10,
        material_type: 0, // Isotropic
        elasticity_modulus: e_mod,
        poissons_ratio: nu,
        ..Default::default()
    };

    let (gpu_forces, element_results) =
        pipeline.execute(4, &gpu_nodes, &gpu_elements, &gpu_material, true);

    let gpu_stress = element_results[0].stress;
    
    // Validate stress
    let eps = 1.0e-3; // Relative tolerance for large values
    assert!((cpu_stress.m11 - gpu_stress[0][0]).abs() / cpu_stress.m11.abs() < eps);
    assert!((cpu_stress.m22 - gpu_stress[1][1]).abs() / cpu_stress.m22.abs() < eps);
    assert!((cpu_stress.m33 - gpu_stress[2][2]).abs() / cpu_stress.m33.abs() < eps);

    // Validate forces
    for i in 0..4 {
        for j in 0..3 {
            let cpu_f = cpu_forces[i][j];
            let gpu_f = gpu_forces[i][j];
            if cpu_f.abs() > 1.0 {
                assert!((cpu_f - gpu_f).abs() / cpu_f.abs() < eps, "Force mismatch at node {}, axis {}: cpu={}, gpu={}", i, j, cpu_f, gpu_f);
            } else {
                assert!((cpu_f - gpu_f).abs() < eps, "Force mismatch at node {}, axis {}: cpu={}, gpu={}", i, j, cpu_f, gpu_f);
            }
        }
    }
}
