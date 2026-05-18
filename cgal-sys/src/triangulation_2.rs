#[cxx::bridge(namespace = "Triangulation2")]
mod ffi {
    unsafe extern "C++" {
        include!("cgal-sys/cpp/pair_utils.h");
        include!("cgal-sys/cpp/triangulation_2.h");
        include!("cgal-sys/cpp/vector_utils.h");

        type EpickPoint2;
        fn create_epick_point_2(x: f64, y: f64) -> UniquePtr<EpickPoint2>;

        #[rust_name = "x"]
        fn get_x_2(p: &EpickPoint2) -> f64;
        #[rust_name = "y"]
        fn get_y_2(p: &EpickPoint2) -> f64;

        type Face2D;
        fn at(self: &Face2D, index: usize) -> &usize;

        type IndexPair;
        #[rust_name = "get_first_index_2"]
        #[namespace = ""]
        fn first(pair: &IndexPair) -> usize;
        #[rust_name = "get_second_index_2"]
        #[namespace = ""]
        fn second(pair: &IndexPair) -> usize;

        type Vertex2D;
        #[rust_name = "get_point_2"]
        #[namespace = ""]
        fn first(vertex: &Vertex2D) -> &EpickPoint2;
        #[rust_name = "get_incident_faces_2"]
        #[namespace = ""]
        fn second(vertex: &Vertex2D) -> &CxxVector<usize>;

        type Data2D;
        fn faces(self: &Data2D) -> &CxxVector<Face2D>;
        fn edges(self: &Data2D) -> &CxxVector<IndexPair>;
        fn vertices(self: &Data2D) -> &CxxVector<Vertex2D>;

        type Constraints;
        #[namespace = ""]
        #[rust_name = "create_constraints_2"]
        fn create_vector(capacity: usize) -> UniquePtr<Constraints>;
        fn reserve(self: Pin<&mut Constraints>, capacity: usize);
        #[namespace = ""]
        #[rust_name = "push_back_2"]
        fn push_back(vec: Pin<&mut Constraints>, constraint: UniquePtr<EpickPoint2>);

        fn triangulate_2(
            constraints: &Constraints,
            aspect_bound: f64,
            size_bound: f64,
        ) -> Result<UniquePtr<Data2D>>;
    }
}

pub use ffi::*;
