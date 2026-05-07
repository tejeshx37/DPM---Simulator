use function::Function;
use nalgebra::{Vector2, Vector3};
use std::{iter::Sum, ops::Add};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum Displacement {
    X(Function),
    Y(Function),
    Z(Function),
    XY(Vector2<Function>),
    XZ(Vector2<Function>),
    YZ(Vector2<Function>),
    XYZ(Vector3<Function>),
}

// Implement Add for Displacement if necessary, or just rely on a simpler macro.
// For now, implementing Add is complex with all 7 variants (49 combinations).
// Let's implement it quickly or simply since it's required by BoundaryCondition.
// Actually, `Vector3<Option<Function>>` is much easier but would break API.
// We'll keep the enum structure and just define Add manually.
impl Add for Displacement {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        // We can just convert both to XYZ format, add them, and convert back.
        let mut x = None;
        let mut y = None;
        let mut z = None;

        let mut add_to = |disp: Self| match disp {
            Self::X(f) => x = Some(match x.take() { Some(f1) => f1 + f, None => f }),
            Self::Y(f) => y = Some(match y.take() { Some(f1) => f1 + f, None => f }),
            Self::Z(f) => z = Some(match z.take() { Some(f1) => f1 + f, None => f }),
            Self::XY(v) => {
                let [[fx, fy]] = v.data.0;
                x = Some(match x.take() { Some(f1) => f1 + fx, None => fx });
                y = Some(match y.take() { Some(f1) => f1 + fy, None => fy });
            }
            Self::XZ(v) => {
                let [[fx, fz]] = v.data.0;
                x = Some(match x.take() { Some(f1) => f1 + fx, None => fx });
                z = Some(match z.take() { Some(f1) => f1 + fz, None => fz });
            }
            Self::YZ(v) => {
                let [[fy, fz]] = v.data.0;
                y = Some(match y.take() { Some(f1) => f1 + fy, None => fy });
                z = Some(match z.take() { Some(f1) => f1 + fz, None => fz });
            }
            Self::XYZ(v) => {
                let [[fx, fy, fz]] = v.data.0;
                x = Some(match x.take() { Some(f1) => f1 + fx, None => fx });
                y = Some(match y.take() { Some(f1) => f1 + fy, None => fy });
                z = Some(match z.take() { Some(f1) => f1 + fz, None => fz });
            }
        };

        add_to(self);
        add_to(rhs);

        match (x, y, z) {
            (Some(fx), Some(fy), Some(fz)) => Self::XYZ(Vector3::new(fx, fy, fz)),
            (Some(fx), Some(fy), None) => Self::XY(Vector2::new(fx, fy)),
            (Some(fx), None, Some(fz)) => Self::XZ(Vector2::new(fx, fz)),
            (None, Some(fy), Some(fz)) => Self::YZ(Vector2::new(fy, fz)),
            (Some(fx), None, None) => Self::X(fx),
            (None, Some(fy), None) => Self::Y(fy),
            (None, None, Some(fz)) => Self::Z(fz),
            (None, None, None) => unreachable!(),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub enum BoundaryCondition {
    #[default]
    Free,
    Force(Vector3<Function>),
    Displacement(Displacement),
}

impl Add for BoundaryCondition {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Free, other) => other,
            (Self::Force(f), Self::Free) => Self::Force(f),
            (Self::Force(f1), Self::Force(f2)) => {
                let [[x1, y1, z1]] = f1.data.0;
                let [[x2, y2, z2]] = f2.data.0;
                Self::Force(Vector3::new(x1 + x2, y1 + y2, z1 + z2))
            }
            (Self::Force(_), Self::Displacement(d)) => Self::Displacement(d),
            (Self::Displacement(d1), Self::Displacement(d2)) => Self::Displacement(d1 + d2),
            (Self::Displacement(d), _) => Self::Displacement(d),
        }
    }
}

impl Sum for BoundaryCondition {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(BoundaryCondition::default(), Add::add)
    }
}
