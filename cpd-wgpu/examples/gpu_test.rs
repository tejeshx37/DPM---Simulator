use cpd_wgpu::{ComputePipeline, GpuElement, GpuMaterial, GpuNode};
use pollster::block_on;

fn main() {
    println!("Initializing GPU two-pass compute pipeline...");
    let pipeline = block_on(ComputePipeline::new()).expect("Failed to create WGPU pipeline");

    // A simple unit tetrahedron: node 1 slightly displaced in X
    let nodes = vec![
        GpuNode {
            initial_position: [0.0, 0.0, 0.0, 0.0],
            position:         [0.0, 0.0, 0.0, 0.0],
            velocity:         [0.0, 0.0, 0.0, 0.0],
            mass:             1.0,
            _padding:         [0.0; 3],
        },
        GpuNode {
            initial_position: [1.0, 0.0, 0.0, 0.0],
            position:         [1.1, 0.0, 0.0, 0.0], // 10 % uniaxial strain in X
            velocity:         [0.0, 0.0, 0.0, 0.0],
            mass:             1.0,
            _padding:         [0.0; 3],
        },
        GpuNode {
            initial_position: [0.0, 1.0, 0.0, 0.0],
            position:         [0.0, 1.0, 0.0, 0.0],
            velocity:         [0.0, 0.0, 0.0, 0.0],
            mass:             1.0,
            _padding:         [0.0; 3],
        },
        GpuNode {
            initial_position: [0.0, 0.0, 1.0, 0.0],
            position:         [0.0, 0.0, 1.0, 0.0],
            velocity:         [0.0, 0.0, 0.0, 0.0],
            mass:             1.0,
            _padding:         [0.0; 3],
        },
    ];

    // One element — stress seed is zero; Pass 1 will compute the real stress.
    let elements = vec![GpuElement {
        node_indices:        [0, 1, 2, 3],
        stress_col0:         [0.0; 4],
        stress_col1:         [0.0; 4],
        stress_col2:         [0.0; 4],
        is_broken:           0,
        strain_energy_bits:  0,
        is_inverted:         0,
        _padding:            0,
    }];

    // Isotropic steel-like material, no failure criteria
    let material = GpuMaterial {
        density:               7800.0,
        damping:               0.01,
        failure_strain_energy: 0.0,
        failure_tensile:       0.0,
        failure_compressive:   0.0,
        material_type:         0,          // isotropic
        elasticity_modulus:    200e9,
        poissons_ratio:        0.3,
        ex: 0.0, ey: 0.0, ez: 0.0,
        nu_xy: 0.0, nu_yx: 0.0,
        nu_yz: 0.0, nu_zy: 0.0,
        nu_zx: 0.0, nu_xz: 0.0,
        g_xy: 0.0, g_yz: 0.0, g_zx: 0.0,
        _pad: [0.0; 3],
    };

    println!("Dispatching two-pass GPU compute...");
    let (node_forces, element_results) =
        pipeline.execute(nodes.len(), &nodes, &elements, &material, true);

    println!("\n=== Node Forces ===");
    for (i, f) in node_forces.iter().enumerate() {
        println!("  Node {:>2}: [{:+.4e}, {:+.4e}, {:+.4e}] N", i, f[0], f[1], f[2]);
    }

    println!("\n=== Element Results (GPU-computed) ===");
    for (i, res) in element_results.iter().enumerate() {
        println!("  Element {:>2}:", i);
        println!("    Stress σ_xx = {:+.4e} Pa", res.stress[0][0]);
        println!("    Stress σ_yy = {:+.4e} Pa", res.stress[1][1]);
        println!("    Stress σ_zz = {:+.4e} Pa", res.stress[2][2]);
        println!("    Strain energy = {:.4e} J", res.strain_energy);
        println!("    Broken = {}", res.is_broken);
    }
}
