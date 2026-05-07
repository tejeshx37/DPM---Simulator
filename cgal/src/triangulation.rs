use cxx::UniquePtr;
use derive_getters::Getters;
use nalgebra::Vector3;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct Face(pub [usize; 4]);

impl From<&cgal_sys::triangulation::Face> for Face {
    fn from(value: &cgal_sys::triangulation::Face) -> Self {
        Self([*value.at(0), *value.at(1), *value.at(2), *value.at(3)])
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct IndexPair(pub usize, pub usize);

impl From<&cgal_sys::triangulation::IndexPair> for IndexPair {
    fn from(value: &cgal_sys::triangulation::IndexPair) -> Self {
        Self(
            cgal_sys::triangulation::get_first_index(value),
            cgal_sys::triangulation::get_second_index(value),
        )
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Getters)]
pub struct Vertex {
    point: Vector3<f32>,
    incident_faces: Box<[usize]>,
}

impl From<&cgal_sys::triangulation::Vertex> for Vertex {
    fn from(value: &cgal_sys::triangulation::Vertex) -> Self {
        let point = cgal_sys::triangulation::get_point(value);
        Self {
            point: Vector3::new(
                cgal_sys::triangulation::x(point) as f32,
                cgal_sys::triangulation::y(point) as f32,
                cgal_sys::triangulation::z(point) as f32,
            ),
            incident_faces: cgal_sys::triangulation::get_incident_faces(value)
                .iter()
                .copied()
                .collect(),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Getters)]
pub struct Data {
    faces: Box<[Face]>,
    edges: Box<[IndexPair]>,
    vertices: Box<[Vertex]>,
}

impl From<UniquePtr<cgal_sys::triangulation::Data>> for Data {
    fn from(value: UniquePtr<cgal_sys::triangulation::Data>) -> Self {
        Self {
            faces: value.faces().iter().map(Into::into).collect(),
            edges: value.edges().iter().map(Into::into).collect(),
            vertices: value.vertices().iter().map(Into::into).collect(),
        }
    }
}

pub fn triangulate(
    constraints: &[Vector3<f64>],
    aspect_bound: f64,
    size_bound: f64,
) -> Result<Data, String> {
    let _lock = crate::lock();
    let constraints = constraints
        .iter()
        .map(|point| cgal_sys::triangulation::create_epick_point(point.x, point.y, point.z))
        .fold(
            cgal_sys::triangulation::create_constraints(constraints.len()),
            |mut vec, point| {
                cgal_sys::triangulation::push_back(vec.pin_mut(), point);
                vec
            },
        );
    cgal_sys::triangulation::triangulate(&constraints, aspect_bound, size_bound)
        .map(Into::into)
        .map_err(|err| err.to_string())
}
