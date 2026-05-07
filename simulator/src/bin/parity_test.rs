use cgal::{PolygonSet, PolygonSetInputKind, RationalPoint, BoundaryId};
use mesh::{Mesh, Callback};
use std::sync::Arc;
use cpd::config::Config;
use cpd::{MaterialProps, IsotropicMaterialProps};
use cpd::computer::{self, AdvanceResult};
use fxhash::FxHashMap;
use cpd::boundary_condition::BoundaryCondition;
use rayon::prelude::*;
use std::time::Duration;

fn main() {
    println!("Initializing GPU...");
    let gpu_pipeline = pollster::block_on(cpd_wgpu::ComputePipeline::new())
        .expect("Failed to initialize GPU pipeline");
    let gpu_pipeline = Arc::new(gpu_pipeline);

    let mut set = PolygonSet::default();
    let sphere = PolygonSetInputKind::Sphere {
        center: RationalPoint::new(0, 0),
        radius: 1.into(),
    };
    set.process_input(&cgal::PolygonSetInput::Join(sphere)).unwrap();
    
    let poly = &set.polygon_with_holes()[0];
    
    // Use tiny mesh: 16 points -> very few elements, fully traceable
    let num_points = 16usize;
    println!("Generating mesh for {} points...", num_points);
    let mesh = Mesh::generate(
        poly, 
        num_points as u32,
        None, 
        1.0, 
        Some(&PolygonSetInputKind::Sphere {
            center: RationalPoint::new(0, 0),
            radius: 1.into(),
        }),
        None,
        Callback::None
    ).unwrap();
    
    let n_nodes = mesh.triangulation_data().vertices().len();
    let n_elems = mesh.triangulation_data().faces().len();
    println!("Mesh generated. Nodes: {}, Elements: {}", n_nodes, n_elems);

    let boundary_conditions: FxHashMap<BoundaryId, BoundaryCondition> = FxHashMap::default();
    let point_boundary_conditions: FxHashMap<usize, BoundaryCondition> = mesh
        .point_id_map()
        .par_iter()
        .map(|(vertex_index, _)| (*vertex_index, BoundaryCondition::Free))
        .collect();

    let unconfigured_computer = computer::unconfigured(
        mesh.triangulation_data(),
        mesh.boundary_point_map(),
        &boundary_conditions,
        &point_boundary_conditions,
    );

    let unconfigured_computer_gpu = computer::unconfigured(
        mesh.triangulation_data(),
        mesh.boundary_point_map(),
        &boundary_conditions,
        &point_boundary_conditions,
    );

    let bulk_props = cpd::BulkMaterialProps::builder()
        .density(1.0)
        .damping(0.0)
        .failure_criteria(cpd::FailureCriteria::default())
        .build();
    let isotropic_props = IsotropicMaterialProps::builder()
        .bulk_props(bulk_props)
        .elasticity_modulus(1.0)
        .elasticity_condition(cpd::ElasticityCondition::ThreeDimensional)
        .poissons_ratio(0.3)
        .build();
    let material_props = MaterialProps::Isotropic(isotropic_props);
    let config = Config::builder()
        .duration(Duration::from_secs(1))
        .time_delta(Duration::from_millis(1))
        .material_props(material_props)
        .build();

    println!("Configuring CPU Computer...");
    let cpu_computer = unconfigured_computer.configure(config.clone());

    println!("Configuring GPU Computer...");
    let mut gpu_computer = unconfigured_computer_gpu.configure(config.clone());
    gpu_computer.set_gpu_pipeline(Some(gpu_pipeline.clone()));

    println!("Running Step 1 on CPU...");
    let AdvanceResult::InProgress(cpu_next) = cpu_computer.advance(true) else { panic!("Expected InProgress") };
    
    println!("Running Step 1 on GPU...");
    let AdvanceResult::InProgress(gpu_next) = gpu_computer.advance(true) else { panic!("Expected InProgress") };

    println!("\n--- Per-node force & position comparison ---");
    let cpu_data = cpu_next.export_data();
    let gpu_data = gpu_next.export_data();
    
    let mut max_diff = 0.0f32;
    let mut max_force_diff = 0.0f32;
    let mut max_diff_node = 0usize;
    
    for (i, (cpu_node, gpu_node)) in cpu_data.nodes().iter().zip(gpu_data.nodes().iter()).enumerate() {
        let pos_diff = (cpu_node.position() - gpu_node.position()).norm();
        let force_diff = (cpu_node.force() - gpu_node.force()).norm();
        if pos_diff > max_diff {
            max_diff = pos_diff;
            max_diff_node = i;
        }
        if force_diff > max_force_diff {
            max_force_diff = force_diff;
        }
        // Print every node since mesh is tiny
        println!(
            "Node {:3}: CPU_force=[{:+.6e},{:+.6e},{:+.6e}]  GPU_force=[{:+.6e},{:+.6e},{:+.6e}]  force_diff={:.3e}  pos_diff={:.3e}",
            i,
            cpu_node.force().x, cpu_node.force().y, cpu_node.force().z,
            gpu_node.force().x, gpu_node.force().y, gpu_node.force().z,
            force_diff, pos_diff
        );
    }

    println!("\n--- All element stress magnitudes ---");
    let mut max_stress = 0.0f32;
    let mut max_stress_diff = 0.0f32;
    for i in 0..n_elems {
        let sc = cpu_data.elements()[i].stress();
        let sg = gpu_data.elements()[i].stress();
        let norm = sc.norm();
        let diff = (sc - sg).norm();
        if norm > max_stress { max_stress = norm; }
        if diff > max_stress_diff { max_stress_diff = diff; }
        
        let el = &cpu_data.elements()[i];
        let n0 = cpu_data.nodes()[el.indices()[0]].position();
        let n1 = cpu_data.nodes()[el.indices()[1]].position();
        let n2 = cpu_data.nodes()[el.indices()[2]].position();
        let n3 = cpu_data.nodes()[el.indices()[3]].position();
        let r = nalgebra::Matrix3::from_columns(&[n1-n0, n2-n0, n3-n0]);
        let det = r.determinant();
        let vol = det.abs() / 6.0;

        if diff > 1e-4 || norm > 0.001 || vol < 1e-6 {
            println!("  Element {:3}: stress_norm={:.4e}  vol={:.4e}  det={:.4e} broken={}", i, norm, vol, det, el.is_broken());
        }
    }
    println!("  Max stress norm: {:.4e}", max_stress);
    println!("  Max stress diff: {:.4e}", max_stress_diff);

    println!("\nMax force diff:    {:.6e}", max_force_diff);
    println!("Max position diff: {:.6e}  (at node {})", max_diff, max_diff_node);

    if max_diff < 1e-4 {
        println!("\nPARITY TEST PASSED!");
    } else {
        println!("\nPARITY TEST FAILED!");
        std::process::exit(1);
    }
}
