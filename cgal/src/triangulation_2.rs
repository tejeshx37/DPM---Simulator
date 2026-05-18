use cxx::UniquePtr;
use derive_getters::Getters;
use nalgebra::Vector3;
use std::collections::HashSet;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct Face2D(pub [usize; 3]);

impl From<&cgal_sys::triangulation_2::Face2D> for Face2D {
    fn from(value: &cgal_sys::triangulation_2::Face2D) -> Self {
        Self([*value.at(0), *value.at(1), *value.at(2)])
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct IndexPair(pub usize, pub usize);

impl From<&cgal_sys::triangulation_2::IndexPair> for IndexPair {
    fn from(value: &cgal_sys::triangulation_2::IndexPair) -> Self {
        Self(
            cgal_sys::triangulation_2::get_first_index_2(value),
            cgal_sys::triangulation_2::get_second_index_2(value),
        )
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Getters)]
pub struct Vertex2D {
    point: Vector3<f32>,
    incident_faces: Box<[usize]>,
}

impl From<&cgal_sys::triangulation_2::Vertex2D> for Vertex2D {
    fn from(value: &cgal_sys::triangulation_2::Vertex2D) -> Self {
        let _lock = crate::lock();
        let point = cgal_sys::triangulation_2::get_point_2(value);
        Self {
            point: Vector3::new(
                cgal_sys::triangulation_2::x(point) as f32,
                cgal_sys::triangulation_2::y(point) as f32,
                0.0,
            ),
            incident_faces: cgal_sys::triangulation_2::get_incident_faces_2(value)
                .iter()
                .copied()
                .collect(),
        }
    }
}

impl Vertex2D {
    pub(crate) fn new(point: Vector3<f32>, incident_faces: Box<[usize]>) -> Self {
        Self {
            point,
            incident_faces,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Getters)]
pub struct Data2D {
    faces: Box<[Face2D]>,
    edges: Box<[IndexPair]>,
    vertices: Box<[Vertex2D]>,
}

impl From<UniquePtr<cgal_sys::triangulation_2::Data2D>> for Data2D {
    fn from(value: UniquePtr<cgal_sys::triangulation_2::Data2D>) -> Self {
        let _lock = crate::lock();
        Self {
            faces: value.faces().iter().map(Into::into).collect(),
            edges: value.edges().iter().map(Into::into).collect(),
            vertices: value.vertices().iter().map(Into::into).collect(),
        }
    }
}

impl Data2D {
    /// Keeps triangles that satisfy `pred`, drops the rest, compacts vertex indices, and
    /// rebuilds edges plus per-vertex `incident_faces`.
    pub fn retain_triangles(&self, mut pred: impl FnMut(&Face2D, &[Vertex2D]) -> bool) -> Self {
        let verts = self.vertices();
        let kept_faces: Vec<Face2D> = self
            .faces()
            .iter()
            .filter(|f| pred(f, verts))
            .copied()
            .collect();

        if kept_faces.is_empty() {
            return Self {
                faces: Box::new([]),
                edges: Box::new([]),
                vertices: Box::new([]),
            };
        }

        let mut used = vec![false; verts.len()];
        for f in &kept_faces {
            for &i in &f.0 {
                used[i] = true;
            }
        }

        let mut old_to_new = vec![usize::MAX; verts.len()];
        let mut next = 0usize;
        for old in 0..verts.len() {
            if used[old] {
                old_to_new[old] = next;
                next += 1;
            }
        }

        let remapped_faces: Box<[Face2D]> = kept_faces
            .into_iter()
            .map(|Face2D(f)| {
                Face2D([
                    old_to_new[f[0]],
                    old_to_new[f[1]],
                    old_to_new[f[2]],
                ])
            })
            .collect();

        let n_new = next;
        let mut incidents: Vec<Vec<usize>> = vec![Vec::new(); n_new];
        for (fi, face) in remapped_faces.iter().enumerate() {
            for &vi in &face.0 {
                incidents[vi].push(fi);
            }
        }

        let new_vertices: Box<[Vertex2D]> = (0..verts.len())
            .filter(|&old| used[old])
            .map(|old| {
                let ni = old_to_new[old];
                Vertex2D::new(
                    *verts[old].point(),
                    std::mem::take(&mut incidents[ni]).into_boxed_slice(),
                )
            })
            .collect();

        let mut edge_set: HashSet<(usize, usize)> = HashSet::new();
        for face in remapped_faces.iter() {
            let v = face.0;
            let pairs = [(0, 1), (0, 2), (1, 2)];
            for &(i, j) in &pairs {
                let a = v[i];
                let b = v[j];
                edge_set.insert((a.min(b), a.max(b)));
            }
        }
        let mut edge_vec: Vec<IndexPair> = edge_set
            .into_iter()
            .map(|(a, b)| IndexPair(a, b))
            .collect();
        edge_vec.sort_by(|e, f| e.0.cmp(&f.0).then(e.1.cmp(&f.1)));

        Self {
            faces: remapped_faces,
            edges: edge_vec.into_boxed_slice(),
            vertices: new_vertices,
        }
    }
}

pub fn triangulate_2(
    constraints: &[Vector3<f64>],
    aspect_bound: f64,
    size_bound: f64,
) -> Result<Data2D, String> {
    let _lock = crate::lock();
    let constraints = constraints
        .iter()
        .map(|point| cgal_sys::triangulation_2::create_epick_point_2(point.x, point.y))
        .fold(
            cgal_sys::triangulation_2::create_constraints_2(constraints.len()),
            |mut vec, point| {
                cgal_sys::triangulation_2::push_back_2(vec.pin_mut(), point);
                vec
            },
        );
    cgal_sys::triangulation_2::triangulate_2(&constraints, aspect_bound, size_bound)
        .map(Into::into)
        .map_err(|err| err.to_string())
}
