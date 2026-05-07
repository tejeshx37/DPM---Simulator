use mesh::{Mesh, Callback};
use cgal::{PolygonSet, PolygonSetInputKind, RationalPoint, PolygonSetInput};

#[test]
fn test_mesh_size() {
    let mut set = PolygonSet::default();
    let sphere = PolygonSetInputKind::Sphere {
        center: RationalPoint::new(0, 0),
        radius: 1.into(),
    };
    set.process_input(&PolygonSetInput::Join(sphere.clone())).unwrap();
    let poly = &set.polygon_with_holes()[0];
    
    let mesh = Mesh::generate(
        poly, 
        512, 
        None,
        1.0,
        None,
        None,
        mesh::Callback::None,
    ).unwrap();
    
    println!("Nodes: {}, Elements: {}", mesh.triangulation_data().vertices().len(), mesh.triangulation_data().faces().len());
    assert!(mesh.triangulation_data().vertices().len() > 0);
}
