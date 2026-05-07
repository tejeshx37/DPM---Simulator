use crate::{curve::Curve, Point};
use cxx::UniquePtr;
use std::{cell::OnceCell, fmt::Debug};

pub struct Polygon {
    pub(crate) curves: Box<[Curve]>,
    centroid: OnceCell<Point>,
    inner: UniquePtr<cgal_sys::Polygon>,
}

impl Debug for Polygon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Polygon")
            .field("curves", &self.curves)
            .finish()
    }
}

impl Clone for Polygon {
    fn clone(&self) -> Self {
        let _lock = crate::lock();
        cgal_sys::clone_polygon(&self.inner).into()
    }
}

impl Drop for Polygon {
    fn drop(&mut self) {
        let _lock = crate::lock();
        let mut inner = cxx::UniquePtr::null();
        std::mem::swap(&mut self.inner, &mut inner);
        drop(inner);
    }
}

impl Polygon {
    pub fn centroid(&self) -> &Point {
        self.centroid
            .get_or_init(|| cgal_sys::centroid(&self.inner).into())
    }
}

impl From<&cgal_sys::Polygon> for Polygon {
    fn from(polygon: &cgal_sys::Polygon) -> Self {
        let _lock = crate::lock();
        cgal_sys::clone_polygon(polygon).into()
    }
}

impl From<UniquePtr<cgal_sys::Polygon>> for Polygon {
    fn from(polygon: UniquePtr<cgal_sys::Polygon>) -> Self {
        let mut curves = Vec::with_capacity(polygon.size() as usize);
        {
            let mut curves_iter = cgal_sys::curve_iterator(&polygon);
            let mut curves_iter = curves_iter.pin_mut();
            while curves_iter.has_next() {
                let curve = curves_iter.as_mut().next();
                curves.push(cgal_sys::clone_x_monotone_curve(curve).into());
            }
        }
        Self {
            curves: curves.into(),
            centroid: OnceCell::new(),
            inner: polygon,
        }
    }
}
