use super::{end_points::EndPoints, macros};
use crate::{num::Algebraic, Point};
use cgal_sys::XMonotoneCurve;
use cxx::UniquePtr;
use std::{
    borrow::Borrow,
    cell::OnceCell,
    fmt::{self, Debug, Formatter},
    ops::Deref,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Horizontal(Data);

impl Horizontal {
    macros::clamp!(x);
    macros::point_at!(x);
}

#[derive(Debug, Clone, PartialEq)]
pub struct Vertical(Data);

impl Vertical {
    macros::clamp!(y);
    macros::point_at!(y);
}

#[derive(Debug, Clone, PartialEq)]
pub struct Oblique(Data);

impl Oblique {
    macros::clamp!(x, y);
    macros::point_at!(x, y);
}

macro_rules! impl_deref {
    ( $($state:ty),* ) => {
        $( impl Deref for $state {
            type Target = Data;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        } )*
    };
}

impl_deref!(Horizontal, Vertical, Oblique);

pub struct Data {
    end_points: EndPoints,
    mid_point: OnceCell<Point>,
    inner: UniquePtr<XMonotoneCurve>,
}

impl Debug for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Data")
            .field("end_points", &self.end_points)
            .finish()
    }
}

impl Clone for Data {
    fn clone(&self) -> Self {
        let _lock = crate::lock();
        cgal_sys::clone_x_monotone_curve(&self.inner).into()
    }
}

impl Drop for Data {
    fn drop(&mut self) {
        let _lock = crate::lock();
        let mut inner = cxx::UniquePtr::null();
        std::mem::swap(&mut self.inner, &mut inner);
        drop(inner);
    }
}

impl PartialEq for Data {
    fn eq(&self, other: &Self) -> bool {
        cgal_sys::equals(&self.inner, &other.inner)
    }
}

impl From<UniquePtr<cgal_sys::XMonotoneCurve>> for Data {
    fn from(ptr: UniquePtr<cgal_sys::XMonotoneCurve>) -> Self {
        let _lock = crate::lock();
        assert!(ptr.is_special_segment());
        Data {
            end_points: (&*ptr).into(),
            mid_point: OnceCell::new(),
            inner: ptr,
        }
    }
}

impl Data {
    fn new(start: &Point, end: &Point) -> Result<Self, String> {
        let _lock = crate::lock();
        cgal_sys::construct_linear_curve(start, end)
            .map(Into::into)
            .map_err(|err| err.to_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LineSegment {
    Horizontal(Horizontal),
    Vertical(Vertical),
    Oblique(Oblique),
}

macro_rules! points {
    ( $data:expr, $n:expr, $t:ident ) => {{
        paste::paste! {
            let start = $data.end_points.start().$t();
            let end = $data.end_points.end().$t();
            let delta = (end - start) / $n;
            (0..=$n)
                .map(|i| start + &(&delta * &Algebraic::from(i)))
                .map(|t| $data.[<clamp_ $t >](&t))
                .map(|t| $data.[<point_at_ $t>](&t).expect("Point exists in line"))
                .collect()
        }
    }};
}

macro_rules! mid_point {
    ( $data:expr, $t:ident ) => {{
        paste::paste! {
            let start = $data.end_points.start().$t();
            let end = $data.end_points.end().$t();
            let mid = (start + end) / Algebraic::from(2);
            $data.[<point_at_ $t>](&mid).expect("Point exists in line")
        }
    }};
}

impl LineSegment {
    pub fn new(start: &Point, end: &Point) -> Result<Self, String> {
        Data::new(start, end).map(Into::into)
    }

    pub fn end_points(&self) -> &EndPoints {
        &self.data().end_points
    }

    pub fn length(&self) -> f64 {
        self.end_points().length()
    }

    fn data(&self) -> &Data {
        match self {
            LineSegment::Horizontal(data) => data,
            LineSegment::Vertical(data) => data,
            LineSegment::Oblique(data) => data,
        }
    }

    pub fn split(&self, n: u32) -> Vec<Point> {
        match n {
            0 => vec![],
            1 => vec![
                self.end_points().start().clone(),
                self.end_points().end().clone(),
            ],
            n => self.split_n(n),
        }
    }

    fn split_n(&self, n: u32) -> Vec<Point> {
        match self {
            LineSegment::Horizontal(data) => {
                points!(data, n, x)
            }
            LineSegment::Vertical(data) => {
                points!(data, n, y)
            }
            LineSegment::Oblique(data) => {
                points!(data, n, x)
            }
        }
    }

    pub fn mid_point(&self) -> &Point {
        match self {
            LineSegment::Horizontal(data) => data.mid_point.get_or_init(|| mid_point!(data, x)),
            LineSegment::Vertical(data) => data.mid_point.get_or_init(|| mid_point!(data, y)),
            LineSegment::Oblique(data) => data.mid_point.get_or_init(|| mid_point!(data, x)),
        }
    }
}

impl From<Data> for LineSegment {
    fn from(data: Data) -> Self {
        let _lock = crate::lock();
        if cgal_sys::is_horizontal(&data.inner) {
            LineSegment::Horizontal(Horizontal(data))
        } else if data.inner.is_vertical() {
            LineSegment::Vertical(Vertical(data))
        } else {
            LineSegment::Oblique(Oblique(data))
        }
    }
}

impl From<UniquePtr<cgal_sys::XMonotoneCurve>> for LineSegment {
    fn from(ptr: UniquePtr<cgal_sys::XMonotoneCurve>) -> Self {
        Data::from(ptr).into()
    }
}

impl Borrow<XMonotoneCurve> for LineSegment {
    fn borrow(&self) -> &XMonotoneCurve {
        &self.data().inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fails_to_build_on_same_point() {
        let origin = Point::new(0, 0);
        assert!(LineSegment::new(&origin, &origin).is_err());
    }

    #[test]
    fn direction_is_mapped_correctly() {
        let start = Point::new(1, 1);
        let end = Point::new(4, 5);
        let line = LineSegment::new(&start, &end).unwrap();
        assert!(matches!(line, LineSegment::Oblique(_)));

        let end = Point::new(4, 1);
        let line = LineSegment::new(&start, &end).unwrap();
        assert!(matches!(line, LineSegment::Horizontal(_)));

        let end = Point::new(1, 4);
        let line = LineSegment::new(&start, &end).unwrap();
        assert!(matches!(line, LineSegment::Vertical(_)));
    }

    #[test]
    fn length_works() {
        let start = Point::new(1, 1);
        let end = Point::new(4, 5);
        let line = LineSegment::new(&start, &end).unwrap();
        assert_eq!(line.length(), 5.0);
    }

    #[test]
    fn split_works() {
        let start = Point::new(1, 1);
        let end = Point::new(6, 1);
        let line = LineSegment::new(&start, &end).unwrap();

        let points = line.split(5);
        assert_eq!(points.len(), 6);
        points.into_iter().enumerate().for_each(|(i, point)| {
            assert_eq!(
                point,
                Point::new(start.x() + &Algebraic::from(i as u32), start.y().clone())
            )
        });
    }
}
