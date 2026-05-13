use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};
use cgal::BoundaryId;

#[derive(Debug, Clone, Default)]
pub struct Geometry3D {
    pub vertices: Vec<[f64; 3]>,
    pub triangles: Vec<[usize; 3]>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PolygonData {
    pub inputs: Vec<cgal::PolygonSetInput>,
    #[serde(skip, default)]
    polygon_set: Arc<OnceLock<cgal::PolygonSet>>,
    #[serde(skip, default)]
    polyhedron_set: Arc<OnceLock<cgal::PolyhedronSet>>,
    #[serde(skip, default)]
    cached_geometry: Arc<OnceLock<Vec<(BoundaryId, Vec<[f64; 2]>)>>>,
    #[serde(skip, default)]
    cached_geometry_3d: Arc<OnceLock<Geometry3D>>,
}

impl PolygonData {
    pub(super) fn new(inputs: Vec<cgal::PolygonSetInput>) -> Self {
        Self {
            inputs,
            polygon_set: Arc::new(OnceLock::new()),
            polyhedron_set: Arc::new(OnceLock::new()),
            cached_geometry: Arc::new(OnceLock::new()),
            cached_geometry_3d: Arc::new(OnceLock::new()),
        }
    }

    pub fn polygon_set(&self) -> &cgal::PolygonSet {
        self.polygon_set.get_or_init(|| {
            let _lock = cgal::lock();
            cgal::PolygonSet::from_inputs(&self.inputs).unwrap_or_else(|err| {
                log::error!("Invalid polygon inputs: {err}");
                cgal::PolygonSet::default()
            })
        })
    }

    pub fn polyhedron_set(&self) -> &cgal::PolyhedronSet {
        self.polyhedron_set.get_or_init(|| {
            let _lock = cgal::lock();
            cgal::PolyhedronSet::from_inputs(&self.inputs).unwrap_or_else(|err| {
                log::error!("Invalid polyhedron inputs: {err}");
                cgal::PolyhedronSet::default()
            })
        })
    }

    pub fn plot_geometry(&self) -> &[(BoundaryId, Vec<[f64; 2]>)] {
        self.cached_geometry.get_or_init(|| {
            let ps = self.polygon_set();
            let _lock = cgal::lock();
            ps.polygon_with_holes()
                .iter()
                .flat_map(cgal::PolygonWithHoles::boundaries_iter)
                .map(|(id, curve)| {
                    let points = match curve {
                        cgal::curve::Curve::Line(line) => vec![
                            (line.end_points().start()).into(),
                            (line.end_points().end()).into(),
                        ],
                        cgal::curve::Curve::Ellipse(arc) => arc.polyline().to_vec(),
                    };
                    (id, points)
                })
                .collect()
        })
    }

    pub fn plot_geometry_3d(&self) -> &Geometry3D {
        self.cached_geometry_3d.get_or_init(|| {
            let phs = self.polyhedron_set();
            if phs.is_empty() {
                return Geometry3D::default();
            }
            let _lock = cgal::lock();
            let mesh = phs.get_mesh();
            
            let pts: Vec<[f64; 3]> = mesh.vertices.iter().map(|v| {
                [v.x, v.y, v.z]
            }).collect();

            let mut tris = Vec::new();
            for chunk in mesh.triangles.chunks_exact(3) {
                let v0 = chunk[0] as usize;
                let v1 = chunk[1] as usize;
                let v2 = chunk[2] as usize;
                tris.push([v0, v1, v2]);
            }

            
            Geometry3D {
                vertices: pts,
                triangles: tris,
            }
        })
    }
}
