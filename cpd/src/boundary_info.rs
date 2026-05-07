use crate::boundary_condition::BoundaryCondition;
use fxhash::FxHashSet;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct BoundaryInfo {
    pub boundary_condition: BoundaryCondition,
    pub node_indices: FxHashSet<usize>,
}
