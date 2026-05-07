use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use cgal::BoundaryId;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PolygonData {
    pub inputs: Vec<cgal::PolygonSetInput>,
    #[serde(skip)]
    polygon_set: OnceLock<cgal::PolygonSet>,
    #[serde(skip)]
    cached_geometry: OnceLock<Vec<(BoundaryId, Vec<[f64; 2]>)>>,
}

impl PolygonData {
    pub(super) fn new(inputs: Vec<cgal::PolygonSetInput>) -> Self {
        Self {
            inputs,
            polygon_set: OnceLock::new(),
            cached_geometry: OnceLock::new(),
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
}
