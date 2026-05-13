use cpd::boundary_condition::BoundaryCondition;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

/// Which 3D axis to test against for a face-plane boundary condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter, Display)]
pub enum Axis3D {
    X,
    Y,
    Z,
}

/// How the node coordinate is compared to the threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter, Display)]
pub enum PlaneComparison {
    #[strum(to_string = "≤ (min face)")]
    LessOrEqual,
    #[strum(to_string = "≥ (max face)")]
    GreaterOrEqual,
    #[strum(to_string = "≈ (near plane)")]
    Approx,
}

/// A boundary condition applied to all mesh nodes satisfying an axis-plane predicate.
///
/// Example: "Fix all nodes where Z ≤ 0.0" — this represents the bottom face of a cube.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacePlaneCondition {
    pub axis: Axis3D,
    pub comparison: PlaneComparison,
    /// The threshold value on the chosen axis.
    pub value: f64,
    /// Tolerance used only for `PlaneComparison::Approx`.
    pub epsilon: f64,
    pub condition: BoundaryCondition,
}

impl Default for FacePlaneCondition {
    fn default() -> Self {
        Self {
            axis: Axis3D::Z,
            comparison: PlaneComparison::LessOrEqual,
            value: 0.0,
            epsilon: 1e-6,
            condition: BoundaryCondition::default(),
        }
    }
}

impl FacePlaneCondition {
    /// Returns `true` if the given 3D node position matches this condition's predicate.
    pub fn matches(&self, pos: [f64; 3]) -> bool {
        let coord = match self.axis {
            Axis3D::X => pos[0],
            Axis3D::Y => pos[1],
            Axis3D::Z => pos[2],
        };
        match self.comparison {
            PlaneComparison::LessOrEqual => coord <= self.value,
            PlaneComparison::GreaterOrEqual => coord >= self.value,
            PlaneComparison::Approx => (coord - self.value).abs() <= self.epsilon,
        }
    }

    /// Returns a human-readable description for the UI.
    pub fn label(&self) -> String {
        match self.comparison {
            PlaneComparison::Approx => {
                format!("{} ≈ {:.3} (±{:.1e})", self.axis, self.value, self.epsilon)
            }
            PlaneComparison::LessOrEqual => {
                format!("{} ≤ {:.3}", self.axis, self.value)
            }
            PlaneComparison::GreaterOrEqual => {
                format!("{} ≥ {:.3}", self.axis, self.value)
            }
        }
    }
}
