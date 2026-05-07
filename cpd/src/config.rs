use super::material;
use derive_getters::Getters;
use nalgebra::Vector3;
use std::time::Duration;
use typed_builder::TypedBuilder;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, TypedBuilder, Getters)]
pub struct Config {
    material_props: material::Props,
    duration: Duration,
    time_delta: Duration,
    #[builder(default)]
    adaptive_time_step: bool,
    #[builder(default)]
    min_time_delta: Option<Duration>,
    #[builder(default)]
    max_time_delta: Option<Duration>,
    #[builder(default=Vector3::zeros())]
    body_force: Vector3<f32>,
}

impl Config {
    pub(crate) fn set_duration(&mut self, duration: Duration) {
        self.duration = duration;
    }

    pub(crate) fn set_time_delta(&mut self, time_delta: Duration) {
        self.time_delta = time_delta;
    }
}
