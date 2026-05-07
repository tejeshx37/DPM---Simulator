use super::BulkProps;
use derive_getters::Getters;
use nalgebra::Matrix3;
use typed_builder::TypedBuilder;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, TypedBuilder, Getters)]
pub struct Props {
    bulk_props: BulkProps,
    elasticity_modulus_x: f32,
    elasticity_modulus_y: f32,
    elasticity_modulus_z: f32,
    poissons_ratio_xy: f32,
    poissons_ratio_yx: f32,
    poissons_ratio_yz: f32,
    poissons_ratio_zy: f32,
    poissons_ratio_zx: f32,
    poissons_ratio_xz: f32,
    shear_modulus_xy: f32,
    shear_modulus_yz: f32,
    shear_modulus_zx: f32,
}

impl Props {
    pub(crate) fn eval_stress(&self, strain: &Matrix3<f32>) -> Matrix3<f32> {
        let s = self.compliance_matrix();
        let d = s.try_inverse().unwrap_or_else(nalgebra::Matrix6::zeros);
        super::stress_vector_to_matrix(d * super::strain_matrix_to_vector(strain))
    }

    fn compliance_matrix(&self) -> nalgebra::Matrix6<f32> {
        nalgebra::matrix![
            1.0 / self.elasticity_modulus_x, -self.poissons_ratio_yx / self.elasticity_modulus_y, -self.poissons_ratio_zx / self.elasticity_modulus_z, 0.0, 0.0, 0.0;
            -self.poissons_ratio_xy / self.elasticity_modulus_x, 1.0 / self.elasticity_modulus_y, -self.poissons_ratio_zy / self.elasticity_modulus_z, 0.0, 0.0, 0.0;
            -self.poissons_ratio_xz / self.elasticity_modulus_x, -self.poissons_ratio_yz / self.elasticity_modulus_y, 1.0 / self.elasticity_modulus_z, 0.0, 0.0, 0.0;
            0.0, 0.0, 0.0, 1.0 / self.shear_modulus_yz, 0.0, 0.0;
            0.0, 0.0, 0.0, 0.0, 1.0 / self.shear_modulus_zx, 0.0;
            0.0, 0.0, 0.0, 0.0, 0.0, 1.0 / self.shear_modulus_xy;
        ]
    }

    pub fn validate(&self) -> Result<(), String> {
        let tol = 1e-4;
        let diff_xy = (self.poissons_ratio_xy / self.elasticity_modulus_x - self.poissons_ratio_yx / self.elasticity_modulus_y).abs();
        let diff_yz = (self.poissons_ratio_yz / self.elasticity_modulus_y - self.poissons_ratio_zy / self.elasticity_modulus_z).abs();
        let diff_zx = (self.poissons_ratio_zx / self.elasticity_modulus_z - self.poissons_ratio_xz / self.elasticity_modulus_x).abs();

        if diff_xy > tol || diff_yz > tol || diff_zx > tol {
            return Err("Symmetry violation: Compliance matrix must be symmetric (v_ij/E_i == v_ji/E_j).".to_string());
        }

        let s = self.compliance_matrix();
        if s.determinant() <= 0.0 {
            return Err("Compliance matrix is not positive-definite (determinant <= 0). Check modulus and Poisson bounds.".to_string());
        }

        Ok(())
    }
}
