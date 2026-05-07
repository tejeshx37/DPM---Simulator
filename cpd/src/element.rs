use super::TimeSeriesValue;
use derive_getters::Getters;
use nalgebra::Matrix3;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Getters)]
pub struct Element {
    pub(super) indices: [usize; 4],
    pub(super) stress_time_series: TimeSeriesValue<Matrix3<f32>>,
    pub(super) strain: Matrix3<f32>,
    pub(super) strain_energy: f32,
    pub(super) is_broken: bool,
    pub(super) is_inverted: bool,
}

impl Element {
    pub fn new(indices: [usize; 4]) -> Self {
        Self {
            indices,
            stress_time_series: TimeSeriesValue::single_default(),
            strain: Matrix3::zeros(),
            strain_energy: 0.0,
            is_broken: false,
            is_inverted: false,
        }
    }

    pub fn stress(&self) -> &Matrix3<f32> {
        self.stress_time_series.latest()
    }

    pub(super) fn reset(&mut self) {
        self.stress_time_series.default_first();
        self.strain = Matrix3::default();
        self.strain_energy = 0.0;
        self.is_broken = false;
        self.is_inverted = false;
    }
}
