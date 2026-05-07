use cgal::{PolygonSet, PolygonSetInputKind, RationalPoint};
use mesh::{Mesh, Callback};
use cpd::computer::{AdvanceResult, unconfigured};
use cpd::config::Config;
use cpd::{BulkMaterialProps, MaterialProps, IsotropicMaterialProps, ElasticityCondition, FailureCriteria};
use fxhash::FxHashMap;
use std::time::Duration;

fn main() {
    let mut set = PolygonSet::default();
    let sphere = PolygonSetInputKind::Sphere {
        center: RationalPoint::new(0, 0),
        radius: 1.into(),
    };
    set.process_input(&cgal::PolygonSetInput::Join(sphere)).unwrap();
    let poly = &set.polygon_with_holes()[0];
    
    println!("Generating mesh...");
    let mesh = Mesh::generate(
        poly, 
        100, 
        None, 
        1.0, 
        Some(&PolygonSetInputKind::Sphere {
            center: RationalPoint::new(0, 0),
            radius: 1.into(),
        }),
        None,
        Callback::None
    ).unwrap();
    
    let triangulation = mesh.triangulation_data();
    println!("Nodes: {}, Elements: {}", triangulation.vertices().len(), triangulation.faces().len());

    let bulk_props = BulkMaterialProps::builder()
        .density(7850.0)
        .damping(0.1)
        .failure_criteria(FailureCriteria::default())
        .build();

    let material_props = MaterialProps::Isotropic(IsotropicMaterialProps::builder()
        .bulk_props(bulk_props)
        .elasticity_modulus(210e9)
        .elasticity_condition(ElasticityCondition::ThreeDimensional)
        .poissons_ratio(0.3)
        .build());

    let config = Config::builder()
        .material_props(material_props)
        .duration(Duration::from_secs(1))
        .time_delta(Duration::from_millis(1))
        .build();
    
    let computer = unconfigured(
        triangulation,
        &mesh.boundary_point_map(),
        &FxHashMap::default(),
        &FxHashMap::default(),
    );
    
    let mut computer = computer.configure(config);
    
    println!("Advancing 10 steps...");
    for i in 0..10 {
        match computer.advance(true) {
            AdvanceResult::InProgress(c) => {
                computer = c;
                let max_coord = computer.export_data().nodes().iter()
                    .map(|n| n.position().x.abs().max(n.position().y.abs()).max(n.position().z.abs()))
                    .fold(0.0f32, f32::max);
                println!("Step {}: Max coord = {}", i, max_coord);
                if max_coord.is_nan() || max_coord > 1e6 {
                    println!("EXPLOSION DETECTED at step {}!", i);
                    return;
                }
            }
            AdvanceResult::Done(_) => break,
        }
    }
    println!("Simulation stable.");
}
