use super::{end_points::EndPoints, macros};
use crate::{num::Algebraic, Point};
use cgal_sys::{EllipseData, XMonotoneCurve};
use cxx::UniquePtr;
use std::{
    borrow::Borrow,
    cell::OnceCell,
    fmt::{self, Debug, Formatter},
    iter,
};

const ARC_LENGTH_PARTITIONS: u32 = 100;
const ARC_ANGLE_DELTA: f64 = 1e-5;
const MID_POINT_ARC_ANGLE_DELTA: f64 = 1e-3;
const POLYLINE_COUNT: usize = 512;

pub type Polyline = Box<[[f64; 2]]>;

pub struct EllipticalArc {
    pub end_points: EndPoints,
    polyline: OnceCell<Polyline>,
    inner: UniquePtr<XMonotoneCurve>,
    length: OnceCell<f64>,
    mid_point: OnceCell<Point>,
    data: UniquePtr<EllipseData>,
}

impl Debug for EllipticalArc {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&cgal_sys::curve_to_string(&self.inner).to_string())
    }
}

impl Clone for EllipticalArc {
    fn clone(&self) -> Self {
        let _lock = crate::lock();
        cgal_sys::clone_x_monotone_curve(&self.inner).into()
    }
}

impl Drop for EllipticalArc {
    fn drop(&mut self) {
        let _lock = crate::lock();
        let mut inner = cxx::UniquePtr::null();
        let mut data = cxx::UniquePtr::null();
        std::mem::swap(&mut self.inner, &mut inner);
        std::mem::swap(&mut self.data, &mut data);
        drop(inner);
        drop(data);
    }
}

impl PartialEq for EllipticalArc {
    fn eq(&self, other: &Self) -> bool {
        let _lock = crate::lock();
        cgal_sys::equals(&self.inner, &other.inner)
    }
}

impl EllipticalArc {
    pub fn center(&self) -> Point {
        let _lock = crate::lock();
        self.data.center().into()
    }

    pub fn a(&self) -> Algebraic {
        let _lock = crate::lock();
        self.data.a().into()
    }

    pub fn b(&self) -> Algebraic {
        let _lock = crate::lock();
        self.data.b().into()
    }

    pub fn polyline(&self) -> &Polyline {
        self.polyline.get_or_init(|| {
            let _lock = crate::lock();
            cgal_sys::polyline_approximation(&self.inner, POLYLINE_COUNT)
                .iter()
                .map(|pair| [cgal_sys::get_x(pair), cgal_sys::get_y(pair)])
                .collect()
        })
    }

    fn integrand(&self) -> impl Fn(f64) -> f64 {
        let a = self.data.a().double_value();
        let b = self.data.b().double_value();
        move |x| {
            let (sin, cos) = x.sin_cos();
            ((a * sin).powi(2) + (b * cos).powi(2)).sqrt()
        }
    }

    pub fn length(&self) -> f64 {
        *self.length.get_or_init(|| {
            simpsons_approximation::compute(
                self.integrand(),
                self.data.angle_start().double_value(),
                self.data.angle_end().double_value(),
                ARC_LENGTH_PARTITIONS,
            )
            .unwrap_or_else(|| panic!("{ARC_LENGTH_PARTITIONS} is a valid partition count"))
            .abs()
        })
    }

    macros::clamp!(x);
    macros::point_at!(x);

    pub fn split(&self, n: u32) -> Vec<Point> {
        match n {
            0 => vec![],
            1 => vec![
                self.end_points.start().clone(),
                self.end_points.end().clone(),
            ],
            n => self.split_n(n),
        }
    }

    fn split_n(&self, n: u32) -> Vec<Point> {
        let angle_start = self.data.angle_start().double_value();
        let angle_end = self.data.angle_end().double_value();

        let range = angle_end - angle_start;
        assert!(range.abs() != 0.0);

        let delta = range.signum() * ARC_ANGLE_DELTA;
        let is_beyond_range = |angle| {
            let l = (angle - angle_start) / range;
            l >= 1.0
        };
        let sub_arc_length = self.length() / (n as f64);
        if sub_arc_length <= 0.0 {
            return vec![self.end_points.start().clone(), self.end_points.end().clone()];
        }
        let a = self.data.a().double_value();
        let h = self.data.center().x().double_value();
        let integrand = self.integrand();
        let intermediate_points_iter = iter::successors(Some(angle_start), |angle| {
            let next = iter::successors(Some((0.0, *angle)), |(arc_length, angle)| {
                if is_beyond_range(*angle) || *arc_length >= sub_arc_length {
                    None
                } else {
                    let arc_delta = (integrand(*angle) * delta).abs();
                    Some((arc_length + arc_delta, angle + delta))
                }
            })
            .map(|(_, angle)| angle)
            .last();

            // Safety check: if we didn't progress, stop the iteration
            if let Some(next_angle) = next {
                if next_angle == *angle {
                    None
                } else {
                    Some(next_angle)
                }
            } else {
                None
            }
        })
        .map(|angle| {
            let x = h + a * angle.cos();
            let x: Algebraic = x.try_into().expect("X is a valid fp");
            let x = self.clamp_x(&x);
            self.point_at_x(&x).expect("Point should exist in curve")
        })
        .skip(1)
        .take((n - 1) as usize);
        iter::once(self.end_points.start().clone())
            .chain(intermediate_points_iter)
            .chain(iter::once(self.end_points.end().clone()))
            .collect()
    }

    pub fn mid_point(&self) -> &Point {
        self.mid_point.get_or_init(|| self.compute_mid_point())
    }

    fn compute_mid_point(&self) -> Point {
        let angle_start = self.data.angle_start().double_value();
        let angle_end = self.data.angle_end().double_value();

        let range = angle_end - angle_start;
        assert!(range.abs() != 0.0);

        let delta = range.signum() * MID_POINT_ARC_ANGLE_DELTA;
        let is_beyond_range = |angle| {
            let l = (angle - angle_start) / range;
            l >= 1.0
        };
        let mid_arc_length = self.length() / 2.0;
        let a = self.data.a().double_value();
        let h = self.data.center().x().double_value();
        let integrand = self.integrand();
        let angle = iter::successors(Some((0.0, angle_start)), |(arc_length, angle)| {
            if is_beyond_range(*angle) || *arc_length >= mid_arc_length {
                None
            } else {
                let arc_delta = (integrand(*angle) * delta).abs();
                Some((arc_length + arc_delta, angle + delta))
            }
        })
        .map(|(_, angle)| angle)
        .last()
        .expect("Mid angle should exist for any arc");
        let x = h + a * angle.cos();
        let x: Algebraic = x.try_into().expect("X is a valid fp");
        let x = self.clamp_x(&x);
        self.point_at_x(&x).expect("Point should exist in curve")
    }
}

impl From<UniquePtr<XMonotoneCurve>> for EllipticalArc {
    fn from(ptr: UniquePtr<XMonotoneCurve>) -> Self {
        let _lock = crate::lock();
        assert!(!ptr.is_special_segment());
        EllipticalArc {
            end_points: (&*ptr).into(),
            data: cgal_sys::get_ellipse_data(&ptr),
            polyline: OnceCell::new(),
            length: OnceCell::new(),
            mid_point: OnceCell::new(),
            inner: ptr,
        }
    }
}

impl Borrow<XMonotoneCurve> for EllipticalArc {
    fn borrow(&self) -> &XMonotoneCurve {
        &self.inner
    }
}

mod simpsons_approximation {
    fn coefficient(i: u32, n: u32) -> u32 {
        if i == 0 || i == n {
            1
        } else if i % 2 == 0 {
            2
        } else {
            4
        }
    }

    pub fn compute<F>(f: F, a: f64, b: f64, partitions: u32) -> Option<f64>
    where
        F: Fn(f64) -> f64,
    {
        if partitions == 0 || partitions % 2 != 0 {
            return None;
        }
        let delta_x: f64 = (b - a) / Into::<f64>::into(partitions);
        let sum: f64 = (0..=partitions)
            .map(|i| (i.into(), coefficient(i, partitions).into()))
            .map(|(i, ci): (f64, f64)| ci * f(a + i * delta_x))
            .sum();
        Some((delta_x * sum) / 3.0)
    }

    #[cfg(test)]
    mod tests {
        use super::compute;
        use std::f64::consts::FRAC_PI_2;

        fn abs_max_err(upper_bound: f64, a: f64, b: f64, partitions: u32) -> Option<f64> {
            if partitions == 0 {
                None
            } else {
                let partitions: f64 = partitions.into();
                let err = upper_bound * ((b - a).powi(5) / (partitions.powi(4) * 180.0));
                Some(err.abs())
            }
        }

        #[test]
        fn returns_none_for_zero_partitions() {
            assert_eq!(compute(|_| 0.0, 0.0, 0.0, 0), None);
        }

        #[test]
        fn returns_none_for_odd_partitions() {
            assert_eq!(compute(|_| 0.0, 0.0, 0.0, 1), None);
        }

        #[test]
        fn integral_is_correct_for_increasing_interval() {
            let partitions = 1000;
            let a = 0.0;
            let b = FRAC_PI_2;
            let exact_integral = 1.0;
            let computed_integral = super::compute(f64::cos, a, b, partitions).unwrap();
            let err_upper_bound = abs_max_err(1.0, a, b, partitions).unwrap();
            let err = (computed_integral - exact_integral).abs();
            assert!(
                err <= err_upper_bound,
                "{err} is beyond upper bound {err_upper_bound}"
            );
        }

        #[test]
        fn integral_is_correct_for_decreasing_interval() {
            let partitions = 1000;
            let a = FRAC_PI_2;
            let b = 0.0;
            let exact_integral = -1.0;
            let computed_integral = super::compute(f64::cos, a, b, partitions).unwrap();
            let err_upper_bound = abs_max_err(1.0, a, b, partitions).unwrap();
            let err = (computed_integral - exact_integral).abs();
            assert!(
                err <= err_upper_bound,
                "{err} is beyond upper bound {err_upper_bound}"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::num::Rational;
    use approx::AbsDiffEq;
    use cgal_sys::Orientation;
    use std::f64::consts::PI;
    use test_case::test_case;

    impl EllipticalArc {
        pub(crate) fn new<T>(
            h: T,
            k: T,
            width: T,
            height: T,
            orientation: Orientation,
            start: &Point,
            end: &Point,
        ) -> Self
        where
            T: Into<Rational>,
        {
            let conic_curve = {
                let mut curve = cgal_sys::construct_conic_curve(
                    &h.into(),
                    &k.into(),
                    &width.into(),
                    &height.into(),
                )
                .unwrap();
                curve.pin_mut().set_endpoints(start, end);
                curve.pin_mut().set_orientation(orientation);
                curve
            };
            let curves = cgal_sys::split_conic_curve(&conic_curve).unwrap();
            curves
                .iter()
                .filter(|curve| cgal_sys::points_eq(curve.source(), start))
                .filter(|curve| cgal_sys::points_eq(curve.target(), end))
                .next()
                .map(cgal_sys::clone_x_monotone_curve)
                .map(EllipticalArc::from)
                .expect("At least one of the curves should match the params")
        }
    }

    fn ramanujan_approximation(a: f64, b: f64) -> f64 {
        let h = ((a - b) / (a + b)).powi(2);
        PI * (a + b) * (1.0 + ((3.0 * h) / (10.0 + (4.0 - 3.0 * h).sqrt())))
    }

    fn start_end(is_ccw: bool, is_upper: bool, r: i32) -> (Point, Point) {
        if is_ccw && is_upper || !is_ccw && !is_upper {
            (Point::new(r, 0), Point::new(-r, 0))
        } else {
            (Point::new(-r, 0), Point::new(r, 0))
        }
    }

    fn test_elliptical_arc(is_ccw: bool, is_upper: bool) -> EllipticalArc {
        let a = 2;
        let b = 1;
        let (start, end) = start_end(is_ccw, is_upper, a);
        EllipticalArc::new(
            0,
            0,
            a * 2,
            b * 2,
            if is_ccw {
                Orientation::COUNTERCLOCKWISE
            } else {
                Orientation::CLOCKWISE
            },
            &start,
            &end,
        )
    }

    #[test_case(true, true)]
    #[test_case(true, false)]
    #[test_case(false, true)]
    #[test_case(false, false)]
    fn length_works_for_elliptical_arc(is_ccw: bool, is_upper: bool) {
        let arc = test_elliptical_arc(is_ccw, is_upper);
        let approx_length =
            ramanujan_approximation(arc.a().double_value(), arc.b().double_value()) / 2.0;
        arc.length().abs_diff_eq(&approx_length, 1e-7);
    }

    fn test_circular_arc(is_ccw: bool, is_upper: bool) -> EllipticalArc {
        let r = 1;
        let (start, end) = start_end(is_ccw, is_upper, r);
        EllipticalArc::new(
            0,
            0,
            r * 2,
            r * 2,
            if is_ccw {
                Orientation::COUNTERCLOCKWISE
            } else {
                Orientation::CLOCKWISE
            },
            &start,
            &end,
        )
    }

    #[test_case(true, true)]
    #[test_case(true, false)]
    #[test_case(false, true)]
    #[test_case(false, false)]
    fn length_works_for_circular_arc(is_ccw: bool, is_upper: bool) {
        let arc = test_circular_arc(is_ccw, is_upper);
        assert_eq!(arc.length(), PI);
    }

    fn expected_splits(n: u32, r: f64, center: &Point, is_ccw: bool, is_upper: bool) -> Vec<Point> {
        let angle = PI / n as f64;
        let y_sign = is_upper.then_some(1).unwrap_or(-1);
        let iter = (0..=n).map(|i| i as f64 * angle).map(|angle| {
            let (sin, cos) = angle.sin_cos();
            let x: Algebraic = (r * cos).try_into().unwrap();
            let y: Algebraic = (r * sin).try_into().unwrap();
            Point::new(center.x() + &x, (center.y() + &y) * y_sign)
        });
        if is_ccw && is_upper || !is_ccw && !is_upper {
            iter.collect()
        } else {
            iter.rev().collect()
        }
    }

    const SPLIT_COUNT: u32 = 5;

    #[test_case(true, true, SPLIT_COUNT)]
    #[test_case(true, false, SPLIT_COUNT)]
    #[test_case(false, true, SPLIT_COUNT)]
    #[test_case(false, false, SPLIT_COUNT)]
    fn split_works_for_circular_arc(is_ccw: bool, is_upper: bool, split_count: u32) {
        let arc = test_circular_arc(is_ccw, is_upper);

        let center = arc.center();
        let r = arc.a().double_value();

        let expected_splits = expected_splits(split_count, r, &center, is_ccw, is_upper);

        println!("{:?}\n{:?}", arc.split(split_count), &expected_splits);

        fn assert_equivalence(lhs: &[Point], rhs: &[Point]) {
            assert_eq!(lhs.len(), rhs.len());
            let epsilon = Algebraic::try_from(0.1).expect("Epsilon is valid fp");
            lhs.iter().zip(rhs).for_each(|(lhs, rhs)| {
                approx::assert_abs_diff_eq!(lhs, rhs, epsilon = epsilon.clone())
            });
        }

        assert_equivalence(&arc.split(split_count), &expected_splits);
    }
}
