mod boundary_average;
pub mod boundary_condition;
mod boundary_info;
pub mod computer;
pub mod config;
mod element;
mod material;
mod node;
mod time_series_value;

pub use boundary_average::{BoundaryAverage, ForceAndDisplacement};
pub(crate) use boundary_info::BoundaryInfo;
use cgal::BoundaryId;
pub use element::Element;
pub use material::{
    isotropic::Props as IsotropicMaterialProps, orthotropic::Props as OrthotropicMaterialProps,
    BulkProps as BulkMaterialProps, ElasticityCondition, FailureCriteria, Props as MaterialProps,
};
pub use node::{Node, NodeData};
pub use time_series_value::{TimeSeriesValue, TimeStampedValue};

use derive_getters::Getters;
use fxhash::FxHashMap;
use nalgebra::Matrix3;

use std::sync::Arc;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Getters)]
pub struct ExportData {
    nodes: Arc<[Node]>,
    elements: Arc<[Element]>,
    boundary_infos: FxHashMap<BoundaryId, BoundaryInfo>,
    #[cfg_attr(feature = "serde", serde(skip))]
    boundary_average_data: Option<BoundaryAverage>,
    config: Option<config::Config>,
    iterations: u128,
    min_stress: Matrix3<f32>,
    max_stress: Matrix3<f32>,
}

impl ExportData {
    pub fn element_vertices(&self, index: usize) -> [[f32; 3]; 4] {
        self.elements[index].indices().map(|i| self.node_position(i))
    }

    pub fn node_position(&self, index: usize) -> [f32; 3] {
        let p = self.nodes[index].position();
        [p.x, p.y, p.z]
    }
}
