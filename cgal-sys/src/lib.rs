#[cxx::bridge]
mod ffi {
    #[repr(i32)]
    #[derive(Debug)]
    enum Orientation {
        CLOCKWISE = -1,
        COLLINEAR = 0,
        COUNTERCLOCKWISE = 1,
    }

    #[repr(i32)]
    #[derive(Debug)]
    enum ComparisonResult {
        SMALLER = -1,
        EQUAL = 0,
        LARGER = 1,
    }

    #[derive(Clone, Copy, Debug, Default)]
    struct Point3D {
        x: f64,
        y: f64,
        z: f64,
    }

    struct Mesh3D {
        vertices: Vec<Point3D>,
        triangles: Vec<u32>,
    }

    unsafe extern "C++" {
        include!("cgal-sys/cpp/curve.h");
        include!("cgal-sys/cpp/kernel.h");
        include!("cgal-sys/cpp/num.h");
        include!("cgal-sys/cpp/pair_utils.h");
        include!("cgal-sys/cpp/point.h");
        include!("cgal-sys/cpp/polygon_set.h");
        include!("cgal-sys/cpp/polygon_with_holes.h");
        include!("cgal-sys/cpp/polygon.h");
        include!("cgal-sys/cpp/vector_utils.h");
        include!("cgal-sys/cpp/polyhedron_set.h");

        type Algebraic;
        #[rust_name = "create_algebraic_from_i32"]
        fn create_algebraic(value: i32) -> UniquePtr<Algebraic>;
        #[rust_name = "create_algebraic_from_u32"]
        fn create_algebraic(value: u32) -> UniquePtr<Algebraic>;
        #[rust_name = "create_algebraic_from_f64"]
        fn create_algebraic(value: f64) -> UniquePtr<Algebraic>;
        #[rust_name = "create_algebraic_from_rational"]
        fn create_algebraic(value: &Rational) -> UniquePtr<Algebraic>;
        #[rust_name = "create_algebraic_from_integer"]
        fn create_algebraic(value: &Integer) -> UniquePtr<Algebraic>;
        #[rust_name = "clone_algebraic"]
        fn create_algebraic(value: &Algebraic) -> UniquePtr<Algebraic>;
        #[rust_name = "abs_algebraic"]
        fn abs(value: &Algebraic) -> UniquePtr<Algebraic>;
        #[rust_name = "double_value"]
        fn doubleValue(self: &Algebraic) -> f64;
        #[rust_name = "algebraic_to_string"]
        fn to_string(value: &Algebraic) -> UniquePtr<CxxString>;
        #[rust_name = "algebraic_from_string"]
        fn from_string(str: &CxxString) -> Result<UniquePtr<Algebraic>>;
        #[rust_name = "add_algebraic"]
        fn add(lhs: &Algebraic, rhs: &Algebraic) -> UniquePtr<Algebraic>;
        #[rust_name = "sub_algebraic"]
        fn sub(lhs: &Algebraic, rhs: &Algebraic) -> UniquePtr<Algebraic>;
        #[rust_name = "mul_algebraic"]
        fn mul(lhs: &Algebraic, rhs: &Algebraic) -> UniquePtr<Algebraic>;
        #[rust_name = "div_algebraic"]
        fn div(lhs: &Algebraic, rhs: &Algebraic) -> UniquePtr<Algebraic>;
        #[rust_name = "neg_algebraic"]
        fn neg(value: &Algebraic) -> UniquePtr<Algebraic>;

        type Rational;
        #[rust_name = "create_rational_from_f64"]
        fn create_rational(value: f64) -> UniquePtr<Rational>;
        #[rust_name = "create_rational_from_i32"]
        fn create_rational(num: i32, den: i32) -> UniquePtr<Rational>;
        #[rust_name = "create_rational_from_integer"]
        fn create_rational(num: &Integer, den: &Integer) -> UniquePtr<Rational>;
        #[rust_name = "clone_rational"]
        fn create_rational(value: &Rational) -> UniquePtr<Rational>;
        #[rust_name = "rational_from_string"]
        fn from_string(str: &CxxString) -> Result<UniquePtr<Rational>>;
        #[rust_name = "rational_to_string"]
        fn to_string(value: &Rational) -> UniquePtr<CxxString>;
        #[rust_name = "rational_to_double"]
        fn rational_to_double(value: &Rational) -> f64;
        #[rust_name = "rational_eq"]
        fn equals(a: &Rational, b: &Rational) -> bool;

        type Integer;
        #[rust_name = "create_integer_from_i32"]
        fn create_integer(value: i32) -> UniquePtr<Integer>;
        #[rust_name = "create_integer_from_u32"]
        fn create_integer(value: u32) -> UniquePtr<Integer>;
        #[rust_name = "clone_integer"]
        fn create_integer(value: &Integer) -> UniquePtr<Integer>;
        #[rust_name = "integer_from_string"]
        fn from_string(str: &CxxString) -> Result<UniquePtr<Integer>>;
        #[rust_name = "integer_to_string"]
        fn to_string(value: &Integer) -> UniquePtr<CxxString>;
        #[rust_name = "integer_eq"]
        fn equals(a: &Integer, b: &Integer) -> bool;
        #[namespace = "CGAL"]
        fn is_zero(value: &Integer) -> bool;
        #[namespace = "CGAL"]
        fn is_negative(value: &Integer) -> bool;
        fn pow_integer(base: &Integer, exp: u32) -> UniquePtr<Integer>;
        #[rust_name = "abs_integer"]
        fn abs(value: &Integer) -> UniquePtr<Integer>;
        #[rust_name = "mul_integer"]
        fn mul(lhs: &Integer, rhs: &Integer) -> UniquePtr<Integer>;

        type Point;
        fn create_point(x: &Algebraic, y: &Algebraic) -> UniquePtr<Point>;
        #[rust_name = "clone_point"]
        fn create_point(point: &Point) -> UniquePtr<Point>;
        fn x(self: &Point) -> &Algebraic;
        fn y(self: &Point) -> &Algebraic;

        type ComparisonResult;
        fn points_eq(first: &Point, second: &Point) -> bool;
        #[namespace = "CGAL"]
        #[rust_name = "compare_algebraic"]
        fn compare(a: &Algebraic, b: &Algebraic) -> ComparisonResult;

        type Orientation;

        type DoublePair;
        #[rust_name = "get_x"]
        fn first(pair: &DoublePair) -> f64;
        #[rust_name = "get_y"]
        fn second(pair: &DoublePair) -> f64;

        type ConicCurve;
        fn construct_conic_curve(
            h: &Rational,
            k: &Rational,
            width: &Rational,
            height: &Rational,
        ) -> Result<UniquePtr<ConicCurve>>;
        fn set_endpoints(self: Pin<&mut ConicCurve>, source: &Point, target: &Point);
        fn set_orientation(self: Pin<&mut ConicCurve>, orientation: Orientation);

        type XMonotoneCurve;
        fn source(self: &XMonotoneCurve) -> &Point;
        fn target(self: &XMonotoneCurve) -> &Point;
        fn is_upper(self: &XMonotoneCurve) -> bool;
        fn polyline_approximation(
            curve: &XMonotoneCurve,
            num_points: usize,
        ) -> UniquePtr<CxxVector<DoublePair>>;
        fn orientation(self: &XMonotoneCurve) -> Orientation;
        fn is_special_segment(self: &XMonotoneCurve) -> bool;
        fn is_horizontal(curve: &XMonotoneCurve) -> bool;
        fn is_vertical(self: &XMonotoneCurve) -> bool;
        fn equals(lhs: &XMonotoneCurve, rhs: &XMonotoneCurve) -> bool;

        fn construct_linear_curve(
            source: &Point,
            target: &Point,
        ) -> Result<UniquePtr<XMonotoneCurve>>;
        fn split_conic_curve(
            curve: &ConicCurve,
        ) -> Result<UniquePtr<CxxVector<XMonotoneCurve>>>;
        fn clone_x_monotone_curve(curve: &XMonotoneCurve) -> UniquePtr<XMonotoneCurve>;

        #[rust_name = "curve_to_string"]
        fn to_string(curve: &XMonotoneCurve) -> UniquePtr<CxxString>;

        type EllipseData;
        fn get_ellipse_data(
            curve: &XMonotoneCurve,
        ) -> UniquePtr<EllipseData>;
        fn center(self: &EllipseData) -> &Point;
        fn a(self: &EllipseData) -> &Algebraic;
        fn b(self: &EllipseData) -> &Algebraic;
        fn angle_start(self: &EllipseData) -> &Algebraic;
        fn angle_end(self: &EllipseData) -> &Algebraic;

        fn point_at_x(
            curve: &XMonotoneCurve,
            x: &Algebraic,
        ) -> Result<UniquePtr<Point>>;
        fn point_at_y(
            curve: &XMonotoneCurve,
            y: &Algebraic,
        ) -> Result<UniquePtr<Point>>;

        type Polygon;
        fn create_polygon() -> UniquePtr<Polygon>;
        #[rust_name = "clone_polygon"]
        fn create_polygon(polygon: &Polygon) -> UniquePtr<Polygon>;
        fn push_back(self: Pin<&mut Polygon>, curve: &XMonotoneCurve) -> Result<()>;
        fn orientation(self: &Polygon) -> Orientation;
        fn reverse_orientation(self: Pin<&mut Polygon>);
        fn size(self: &Polygon) -> u32;
        fn centroid(polygon: &Polygon) -> UniquePtr<Point>;

        type CurveIterator<'a>;
        fn curve_iterator(polygon: &Polygon) -> UniquePtr<CurveIterator<'_>>;
        fn has_next(self: &CurveIterator) -> bool;
        fn next<'a>(self: Pin<&'a mut CurveIterator>) -> &'a XMonotoneCurve;

        type PolygonWithHoles;
        fn create_polygon_with_holes() -> UniquePtr<PolygonWithHoles>;
        #[rust_name = "clone_polygon_with_holes"]
        fn create_polygon_with_holes(polygon: &PolygonWithHoles) -> UniquePtr<PolygonWithHoles>;
        fn outer_boundary(self: &PolygonWithHoles) -> &Polygon;
        fn number_of_holes(self: &PolygonWithHoles) -> u32;

        type HoleIterator<'a>;
        fn hole_iterator(polygon: &PolygonWithHoles) -> UniquePtr<HoleIterator<'_>>;
        fn has_next(self: &HoleIterator) -> bool;
        fn next<'a>(self: Pin<&'a mut HoleIterator>) -> &'a Polygon;

        type PolyhedronSet3;
        fn create_polyhedron_set() -> UniquePtr<PolyhedronSet3>;
        #[rust_name = "clone_polyhedron_set"]
        fn create_polyhedron_set_clone(other: &PolyhedronSet3) -> UniquePtr<PolyhedronSet3>;
        fn is_empty(self: &PolyhedronSet3) -> bool;
        fn is_valid(self: &PolyhedronSet3) -> bool;
        fn join(self: Pin<&mut PolyhedronSet3>, other: &PolyhedronSet3) -> Result<()>;
        fn difference(self: Pin<&mut PolyhedronSet3>, other: &PolyhedronSet3) -> Result<()>;
        fn intersection(self: Pin<&mut PolyhedronSet3>, other: &PolyhedronSet3) -> Result<()>;
        
        fn get_vertices(set: &PolyhedronSet3) -> Result<UniquePtr<CxxVector<Point3D>>>;
        fn get_triangles(set: &PolyhedronSet3) -> Result<UniquePtr<CxxVector<u32>>>;
        fn get_mesh(set: &PolyhedronSet3) -> Result<Mesh3D>;

        fn create_cube(cx: &Rational, cy: &Rational, cz: &Rational, size: &Rational) -> UniquePtr<PolyhedronSet3>;
        fn create_cuboid(cx: &Rational, cy: &Rational, cz: &Rational, width: &Rational, height: &Rational, depth: &Rational) -> UniquePtr<PolyhedronSet3>;
        fn create_approximated_sphere(cx: &Rational, cy: &Rational, cz: &Rational, radius: &Rational, num_lat: u32, num_lon: u32) -> UniquePtr<PolyhedronSet3>;
        fn create_approximated_cone(cx: &Rational, cy: &Rational, cz: &Rational, radius: &Rational, height: &Rational, num_segments: u32) -> UniquePtr<PolyhedronSet3>;
        fn create_approximated_cylinder(cx: &Rational, cy: &Rational, cz: &Rational, radius: &Rational, height: &Rational, num_segments: u32) -> UniquePtr<PolyhedronSet3>;
        fn create_extruded_polygon(pts_x: &[f64], pts_y: &[f64], height: &Rational) -> UniquePtr<PolyhedronSet3>;

        type PolygonSet;
        fn create_polygon_set() -> UniquePtr<PolygonSet>;
        #[rust_name = "clone_polygon_set"]
        fn create_polygon_set(polygon_set: &PolygonSet) -> UniquePtr<PolygonSet>;
        fn insert(self: Pin<&mut PolygonSet>, polygon: &Polygon) -> Result<()>;
        fn join(self: Pin<&mut PolygonSet>, polygon: &Polygon) -> Result<()>;
        fn difference(self: Pin<&mut PolygonSet>, polygon: &Polygon) -> Result<()>;
        fn split_curve(
            polygon_set: Pin<&mut PolygonSet>,
            ref_curve: &XMonotoneCurve,
            point: &Point,
        ) -> Result<()>;
        fn is_empty(self: &PolygonSet) -> bool;
        fn polygon_with_holes(polygon_set: &PolygonSet) -> UniquePtr<CxxVector<PolygonWithHoles>>;
        fn clear(self: Pin<&mut PolygonSet>);
    }
}

pub use ffi::*;
pub mod triangulation;
