mod elliptical_arc;
mod end_points;
mod line_segment;

use crate::Point;
use cgal_sys::XMonotoneCurve;
use cxx::UniquePtr;
pub use elliptical_arc::EllipticalArc;
pub use end_points::EndPoints;
pub use line_segment::LineSegment;
use std::{borrow::Borrow, fmt::Debug};

mod macros {
    macro_rules! clamp {
        ($($t:ident),*) => {
            $( paste::paste! {
                pub fn [<clamp_ $t >](&self, $t: &Algebraic) -> Algebraic {
                    let t1 = self.end_points.start().$t();
                    let t2 = self.end_points.end().$t();
                    let min = t1.min(t2);
                    let max = t1.max(t2);
                    if $t < min {
                        min.clone()
                    } else if $t > max {
                        max.clone()
                    } else {
                        $t.clone()
                    }
                }
            } )*
        };
    }

    macro_rules! point_at {
        ($($t:ident),*) => {
            $( paste::paste! {
                pub fn [<point_at_ $t>](&self, $t: &Algebraic) -> Result<Point, String> {
                    if self.end_points.start().$t() == $t {
                        Ok(self.end_points.start().clone())
                    } else if self.end_points.end().$t() == $t {
                        Ok(self.end_points.end().clone())
                    } else {
                        cgal_sys::[<point_at_ $t>](&self.inner, &$t)
                            .map(Into::into)
                            .map_err(|err| err.to_string())
                    }
                }
            } )*
        };
    }

    pub(crate) use clamp;
    pub(crate) use point_at;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Curve {
    Line(LineSegment),
    Ellipse(EllipticalArc),
}

impl Curve {
    pub fn length(&self) -> f64 {
        let _lock = crate::lock();
        match self {
            Curve::Line(line) => line.length(),
            Curve::Ellipse(arc) => arc.length(),
        }
    }

    pub fn split(&self, n: u32) -> Vec<Point> {
        let _lock = crate::lock();
        match self {
            Curve::Line(line) => line.split(n),
            Curve::Ellipse(arc) => arc.split(n),
        }
    }

    pub fn end_points(&self) -> &EndPoints {
        let _lock = crate::lock();
        match self {
            Curve::Line(line) => line.end_points(),
            Curve::Ellipse(arc) => &arc.end_points,
        }
    }

    pub fn mid_point(&self) -> &Point {
        let _lock = crate::lock();
        match self {
            Curve::Line(line) => line.mid_point(),
            Curve::Ellipse(arc) => arc.mid_point(),
        }
    }
}

impl From<UniquePtr<XMonotoneCurve>> for Curve {
    fn from(ptr: UniquePtr<XMonotoneCurve>) -> Self {
        if ptr.is_special_segment() {
            Curve::Line(LineSegment::from(ptr))
        } else {
            Curve::Ellipse(EllipticalArc::from(ptr))
        }
    }
}

impl Borrow<XMonotoneCurve> for Curve {
    fn borrow(&self) -> &XMonotoneCurve {
        match self {
            Curve::Line(line) => line.borrow(),
            Curve::Ellipse(arc) => arc.borrow(),
        }
    }
}
