use cgal::{PolygonSet, PolygonSetInput, PolygonSetInputKind, RationalPoint};
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
            let (nodes, faces) = match m.triangulation_data() {
                cgal::TriangulationDataRef::TwoD(t) => (t.vertices().len(), t.faces().len()),
                cgal::TriangulationDataRef::ThreeD(t) => (t.vertices().len(), t.faces().len()),
            };
            println!("Nodes: {}", nodes);
            println!("Elements: {}", faces);
        },
        Err(e) => {
            println!("Error generating mesh: {}", e);
        }
    }
}
