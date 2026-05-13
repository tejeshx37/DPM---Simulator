use crate::{curve::Curve, polygon::Polygon};
use cxx::UniquePtr;
use std::{fmt::Debug, ops::Deref};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct HoleId(pub(crate) usize);

impl Deref for HoleId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct CurveId(pub(crate) usize);

impl Deref for CurveId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum BoundaryId {
    OuterBoundary(CurveId),
    Hole(HoleId, CurveId),
}

impl BoundaryId {
    pub fn curve_id(self) -> CurveId {
        match self {
            BoundaryId::OuterBoundary(curve_id) => curve_id,
            BoundaryId::Hole(_, curve_id) => curve_id,
        }
    }
}

pub struct PolygonWithHoles {
    outer_boundary: Polygon,
    holes: Box<[Polygon]>,
    inner: UniquePtr<cgal_sys::PolygonWithHoles>,
}

impl Debug for PolygonWithHoles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PolygonWithHoles")
            .field("outer_boundary", &self.outer_boundary)
            .field("holes", &self.holes)
            .finish()
    }
}

impl Clone for PolygonWithHoles {
    fn clone(&self) -> Self {
        let _lock = crate::lock();
        cgal_sys::clone_polygon_with_holes(&self.inner).into()
    }
}

impl From<&cgal_sys::PolygonWithHoles> for PolygonWithHoles {
    fn from(polygon: &cgal_sys::PolygonWithHoles) -> Self {
        let _lock = crate::lock();
        cgal_sys::clone_polygon_with_holes(polygon).into()
    }
}

impl From<UniquePtr<cgal_sys::PolygonWithHoles>> for PolygonWithHoles {
    fn from(polygon: UniquePtr<cgal_sys::PolygonWithHoles>) -> Self {
        let _lock = crate::lock();
        let outer_boundary = polygon.outer_boundary().into();
        let mut holes = Vec::with_capacity(polygon.number_of_holes() as usize);
        {
            let mut holes_iter = cgal_sys::hole_iterator(&polygon);
            let mut holes_iter = holes_iter.pin_mut();
            while holes_iter.has_next() {
                let hole = holes_iter.as_mut().next();
                holes.push(hole.into());
            }
        }
        Self {
            outer_boundary,
            holes: holes.into(),
            inner: polygon,
        }
    }
}

impl PolygonWithHoles {
    pub fn outer_boundaries(&self) -> impl Iterator<Item = (BoundaryId, &Curve)> {
        self.outer_boundary
            .curves
            .iter()
            .enumerate()
            .map(|(curve_id, curve)| (BoundaryId::OuterBoundary(CurveId(curve_id)), curve))
    }

    pub fn hole_ids(&self) -> impl Iterator<Item = HoleId> + '_ {
        self.holes.iter().enumerate().map(|(id, _)| HoleId(id))
    }

    pub fn hole_boundaries(&self, hole_id: HoleId) -> impl Iterator<Item = (BoundaryId, &Curve)> {
        self.hole_with_id(hole_id)
            .curves
            .iter()
            .enumerate()
            .map(move |(curve_id, curve)| (BoundaryId::Hole(hole_id, CurveId(curve_id)), curve))
    }

    pub fn hole_with_id(&self, hole_id: HoleId) -> &Polygon {
        &self.holes[hole_id.0]
    }

    pub fn boundaries_iter(&self) -> impl Iterator<Item = (BoundaryId, &Curve)> {
        self.outer_boundaries().chain(
            self.hole_ids()
                .flat_map(|hole_id| self.hole_boundaries(hole_id)),
        )
    }

    pub fn boundary_with_id(&self, id: &BoundaryId) -> &Curve {
        match id {
            BoundaryId::OuterBoundary(curve_id) => &self.outer_boundary.curves[curve_id.0],
            BoundaryId::Hole(hole_id, curve_id) => &self.holes[hole_id.0].curves[curve_id.0],
        }
    }
}

impl Drop for PolygonWithHoles {
    fn drop(&mut self) {
        let _lock = crate::lock();
        let mut inner = UniquePtr::null();
        std::mem::swap(&mut self.inner, &mut inner);
        drop(inner);
    }
}
