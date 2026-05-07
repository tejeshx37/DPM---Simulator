use crate::Point;
use derive_getters::Getters;
use std::cell::OnceCell;

#[derive(Debug, Clone, Getters)]
pub struct EndPoints {
    start: Point,
    end: Point,
    #[getter(skip)]
    length: OnceCell<f64>,
}

impl PartialEq for EndPoints {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}

impl From<&cgal_sys::XMonotoneCurve> for EndPoints {
    fn from(value: &cgal_sys::XMonotoneCurve) -> Self {
        Self::new(value.source().into(), value.target().into())
    }
}

impl EndPoints {
    pub fn new(start: Point, end: Point) -> Self {
        Self {
            start,
            end,
            length: OnceCell::new(),
        }
    }

    pub fn length(&self) -> f64 {
        *self.length.get_or_init(|| {
            let len_sqr = (self.end.x().double_value() - self.start.x().double_value()).powi(2)
                + (self.end.y().double_value() - self.start.y().double_value()).powi(2);
            len_sqr.sqrt()
        })
    }
}

#[cfg(test)]
mod test_impls {
    use super::EndPoints;
    use crate::num::Algebraic;
    use approx::AbsDiffEq;

    impl AbsDiffEq for EndPoints {
        type Epsilon = Algebraic;

        fn default_epsilon() -> Self::Epsilon {
            Algebraic::default_epsilon()
        }

        fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
            self.start.abs_diff_eq(&other.start, epsilon.clone())
                && self.end.abs_diff_eq(&other.end, epsilon)
        }
    }
}
