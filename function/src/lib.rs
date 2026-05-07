pub mod piecewise_linear;

use piecewise_linear::PiecewiseLinear;
use std::ops::Add;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Clone)]
pub struct SuperposedFunctions {
    functions: Vec<Function>,
}

impl SuperposedFunctions {
    pub fn scale_amplitude(self, scale: f32) -> Self {
        Self {
            functions: self
                .functions
                .into_iter()
                .map(|f| f.scale_amplitude(scale))
                .collect(),
        }
    }

    pub fn of(&self, x: f32) -> Option<f32> {
        self.functions.iter().map(|f| f.of(x)).sum()
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Clone)]
pub enum Function {
    Piecewise(PiecewiseLinear),
    Superposed(SuperposedFunctions),
}

impl Function {
    pub fn scale_amplitude(self, scale: f32) -> Self {
        match self {
            Self::Piecewise(f) => Self::Piecewise(f.scale_amplitude(scale)),
            Self::Superposed(s) => Self::Superposed(s.scale_amplitude(scale)),
        }
    }

    pub fn of(&self, x: f32) -> Option<f32> {
        match self {
            Self::Piecewise(f) => f.of(x),
            Self::Superposed(s) => s.of(x),
        }
    }
}

impl Add for Function {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Superposed(mut s1), Self::Superposed(s2)) => {
                s1.functions.extend(s2.functions);
                Self::Superposed(s1)
            }
            (Self::Superposed(mut s), f) => {
                s.functions.push(f);
                Self::Superposed(s)
            }
            (f1, f2) => Self::Superposed(SuperposedFunctions {
                functions: vec![f1, f2],
            }),
        }
    }
}
