pub mod curve;
pub mod num;
mod point;
mod polygon;
mod polygon_set;
mod polygon_with_holes;
pub mod polyhedron_set;
pub mod triangulation;

use std::sync::Mutex;
pub static CGAL_LOCK: Mutex<()> = Mutex::new(());

use std::cell::Cell;
thread_local! {
    static LOCK_DEPTH: Cell<usize> = Cell::new(0);
}

#[allow(dead_code)]
pub struct CgalLockGuard(Option<std::sync::MutexGuard<'static, ()>>);

impl Drop for CgalLockGuard {
    fn drop(&mut self) {
        LOCK_DEPTH.with(|d| d.set(d.get() - 1));
    }
}

pub fn lock() -> CgalLockGuard {
    let depth = LOCK_DEPTH.with(|d| {
        let old = d.get();
        d.set(old + 1);
        old
    });
    if depth == 0 {
        CgalLockGuard(Some(CGAL_LOCK.lock().unwrap()))
    } else {
        CgalLockGuard(None)
    }
}

pub use point::Point;
pub use polygon::Polygon;
pub use polygon_set::{
    Coordinate, Input as PolygonSetInput, InputKind as PolygonSetInputKind, PolygonSet,
    RationalPoint, RationalPoint3,
};
pub use polygon_with_holes::{BoundaryId, PolygonWithHoles};
pub use polyhedron_set::PolyhedronSet;
