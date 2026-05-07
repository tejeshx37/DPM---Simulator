use crate::num::Algebraic;
use cxx::UniquePtr;
use derive_getters::Getters;
use nalgebra::Vector2;
use std::{
    fmt::{self, Debug, Display, Formatter},
    ops::Deref,
};

#[derive(Getters)]
pub struct Point {
    x: Algebraic,
    y: Algebraic,
    #[getter(skip)]
    inner: UniquePtr<cgal_sys::Point>,
}

impl Point {
    pub fn new<T>(x: T, y: T) -> Self
    where
        T: Into<Algebraic>,
    {
        let _lock = crate::lock();
        cgal_sys::create_point(&x.into(), &y.into()).into()
    }
}

impl Drop for Point {
    fn drop(&mut self) {
        let _lock = crate::lock();
        let mut inner = UniquePtr::null();
        std::mem::swap(&mut self.inner, &mut inner);
        drop(inner);
    }
}

impl From<UniquePtr<cgal_sys::Point>> for Point {
    fn from(ptr: UniquePtr<cgal_sys::Point>) -> Self {
        Self {
            x: ptr.x().into(),
            y: ptr.y().into(),
            inner: ptr,
        }
    }
}

impl From<&cgal_sys::Point> for Point {
    fn from(value: &cgal_sys::Point) -> Self {
        let _lock = crate::lock();
        cgal_sys::clone_point(value).into()
    }
}

impl Clone for Point {
    fn clone(&self) -> Self {
        let _lock = crate::lock();
        cgal_sys::clone_point(self).into()
    }
}

impl Deref for Point {
    type Target = cgal_sys::Point;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Point> for UniquePtr<cgal_sys::Point> {
    fn from(value: Point) -> Self {
        (&value).into()
    }
}

impl From<&Point> for UniquePtr<cgal_sys::Point> {
    fn from(value: &Point) -> Self {
        let _lock = crate::lock();
        cgal_sys::clone_point(&value.inner)
    }
}

impl From<Point> for [f64; 2] {
    fn from(value: Point) -> Self {
        (&value).into()
    }
}

impl From<&Point> for [f64; 2] {
    fn from(value: &Point) -> Self {
        [value.x.double_value(), value.y.double_value()]
    }
}

impl From<Point> for Vector2<f64> {
    fn from(point: Point) -> Self {
        (&point).into()
    }
}

impl From<&Point> for Vector2<f64> {
    fn from(point: &Point) -> Self {
        let [x, y] = point.into();
        Vector2::new(x, y)
    }
}

impl Debug for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Point")
            .field("x", &self.x)
            .field("y", &self.y)
            .finish()
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        let _lock = crate::lock();
        cgal_sys::points_eq(self, other)
    }
}

#[cfg(test)]
mod test_impls {
    use super::{Algebraic, Point};
    use approx::AbsDiffEq;

    impl AbsDiffEq for Point {
        type Epsilon = Algebraic;

        fn default_epsilon() -> Self::Epsilon {
            Algebraic::default_epsilon()
        }

        fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
            self.x.abs_diff_eq(&other.x, epsilon.clone()) && self.y.abs_diff_eq(&other.y, epsilon)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_works() {
        let p = Point::new(1, 2);
        assert_eq!(p.x, Algebraic::from(1));
        assert_eq!(p.y, Algebraic::from(2));
    }

    #[test]
    fn partial_eq_works() {
        let point = Point::new(-2, 0);
        assert_eq!(point, Point::new(-2, 0));
        assert_ne!(point, Point::new(0, 0));
    }
}
