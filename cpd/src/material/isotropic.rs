use super::{BulkProps, ElasticityCondition};
use derive_getters::Getters;
use nalgebra::{Matrix3, Matrix6};
use typed_builder::TypedBuilder;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, TypedBuilder, Getters)]
pub struct Props {
    bulk_props: BulkProps,
    elasticity_modulus: f32,
    elasticity_condition: ElasticityCondition,
    poissons_ratio: f32,
}

impl Props {
    pub(crate) fn eval_stress(&self, strain: &Matrix3<f32>) -> Matrix3<f32> {
        isotropic_3d(self.elasticity_modulus, self.poissons_ratio, strain)
    }
}

fn isotropic_3d_matrix(v: f32) -> Matrix6<f32> {
    nalgebra::matrix![
        1.0 - v, v, v, 0.0, 0.0, 0.0;
        v, 1.0 - v, v, 0.0, 0.0, 0.0;
        v, v, 1.0 - v, 0.0, 0.0, 0.0;
        0.0, 0.0, 0.0, (1.0 - v * 2.0) / 2.0, 0.0, 0.0;
        0.0, 0.0, 0.0, 0.0, (1.0 - v * 2.0) / 2.0, 0.0;
        0.0, 0.0, 0.0, 0.0, 0.0, (1.0 - v * 2.0) / 2.0;
    ]
}

fn isotropic_3d(e: f32, v: f32, strain: &Matrix3<f32>) -> Matrix3<f32> {
    // Numerical Safeguard: Clamp Poisson's ratio to prevent singularity at 0.5
    let v = v.clamp(0.0, 0.499);
    super::stress_vector_to_matrix(
        (e / ((1.0 + v) * (1.0 - 2.0 * v)))
            * isotropic_3d_matrix(v)
            * super::strain_matrix_to_vector(strain),
    )
}
