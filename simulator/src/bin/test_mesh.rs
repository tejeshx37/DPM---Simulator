use cgal::{PolygonSet, PolygonSetInputKind, RationalPoint};
use mesh::{Mesh, Callback};

fn main() {
    let mut set = PolygonSet::default();
    let sphere = PolygonSetInputKind::Sphere {
        center: RationalPoint::new(0, 0),
        radius: 1.into(),
    };
    set.process_input(&cgal::PolygonSetInput::Join(sphere)).unwrap();
    
    let poly = &set.polygon_with_holes()[0];
    
    println!("Generating mesh for 512 points...");
    let mesh = Mesh::generate(
        poly, 
        512, 
        None, 
        1.0, 
        Some(&PolygonSetInputKind::Sphere {
            center: RationalPoint::new(0, 0),
            radius: 1.into(),
        }),
        None,
        Callback::None
    ).unwrap();
    
    println!("Mesh generated.");
    println!("Nodes: {}", mesh.triangulation_data().vertices().len());
    println!("Elements: {}", mesh.triangulation_data().faces().len());
}
