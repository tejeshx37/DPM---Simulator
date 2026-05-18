#[cxx::bridge(namespace = "Triangulation")]
mod ffi {
    unsafe extern "C++" {
        include!("cgal-sys/cpp/pair_utils.h");
        include!("cgal-sys/cpp/triangulation.h");
        include!("cgal-sys/cpp/vector_utils.h");

        #[namespace = ""]
        type Point = crate::Point;

        type EpickPoint;
        fn create_epick_point(x: f64, y: f64, z: f64) -> UniquePtr<EpickPoint>;

        #[rust_name = "x"]
        fn get_x(p: &EpickPoint) -> f64;
        #[rust_name = "y"]
        fn get_y(p: &EpickPoint) -> f64;
        #[rust_name = "z"]
        fn get_z(p: &EpickPoint) -> f64;

        type Face;
        fn at(self: &Face, index: usize) -> &usize;

        type IndexPair;
        #[rust_name = "get_first_index"]
        #[namespace = ""]
        fn first(pair: &IndexPair) -> usize;
        #[rust_name = "get_second_index"]
        #[namespace = ""]
        fn second(pair: &IndexPair) -> usize;

        type Vertex;
        #[rust_name = "get_point"]
        #[namespace = ""]
        fn first_ref(vertex: &Vertex) -> &EpickPoint;
        #[rust_name = "get_incident_faces"]
        #[namespace = ""]
        fn second_ref(vertex: &Vertex) -> &CxxVector<usize>;

        type Data;
        fn faces(self: &Data) -> &CxxVector<Face>;
        fn edges(self: &Data) -> &CxxVector<IndexPair>;
        fn vertices(self: &Data) -> &CxxVector<Vertex>;

        type Constraints;
        #[namespace = ""]
        #[rust_name = "create_constraints"]
        fn create_vector(capacity: usize) -> UniquePtr<Constraints>;
        fn reserve(self: Pin<&mut Constraints>, capacity: usize);
        #[namespace = ""]
        fn push_back(vec: Pin<&mut Constraints>, constraint: UniquePtr<EpickPoint>);

        fn triangulate(
            constraints: &Constraints,
            aspect_bound: f64,
            size_bound: f64,
        ) -> Result<UniquePtr<Data>>;
    }
}

pub use ffi::*;
