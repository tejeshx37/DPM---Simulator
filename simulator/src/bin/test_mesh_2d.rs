use cgal::{PolygonSet, PolygonSetInputKind, RationalPoint};
use mesh::{Mesh, Callback};

fn main() {
    let mut set = PolygonSet::default();
    let rect = PolygonSetInputKind::LinearPolygon(vec![
        RationalPoint::new(0, 0),
        RationalPoint::new(10, 0),
        RationalPoint::new(10, 10),
        RationalPoint::new(0, 10),
    ]);
    set.process_input(&cgal::PolygonSetInput::Join(rect)).unwrap();
    
    let poly = &set.polygon_with_holes()[0];
    
    println!("Generating mesh for 512 points...");
    let mesh = Mesh::generate(
        poly, 
        512, 
        None, 
        0.0, 
        None,
        None,
        Callback::None
    );
    
    match mesh {
        Ok(m) => {
            println!("Mesh generated.");
            let nodes = m.triangulation_data().vertices().len();
            let faces = m.triangulation_data().faces().len();
            println!("Nodes: {}", nodes);
            println!("Elements: {}", faces);
        },
        Err(e) => {
            println!("Error generating mesh: {}", e);
        }
    }
}
