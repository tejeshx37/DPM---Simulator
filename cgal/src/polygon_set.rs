use crate::{
    curve::{Curve, LineSegment},
    num::Algebraic,
    num::Rational,
    polygon_with_holes::BoundaryId,
    Point, PolygonWithHoles,
};
use cgal_sys::{Polygon, XMonotoneCurve};
use cxx::UniquePtr;
use std::{borrow::Borrow, fmt::Debug};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RationalPoint {
    pub x: Rational,
    pub y: Rational,
}

impl RationalPoint {
    pub fn new<T>(x: T, y: T) -> Self
    where
        T: Into<Rational>,
    {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }
}

impl From<RationalPoint> for Point {
    fn from(value: RationalPoint) -> Self {
        (&value).into()
    }
}

impl From<&RationalPoint> for Point {
    fn from(value: &RationalPoint) -> Self {
        Self::new(&value.x, &value.y)
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RationalPoint3 {
    pub x: Rational,
    pub y: Rational,
    pub z: Rational,
}

impl RationalPoint3 {
    pub fn new<T>(x: T, y: T, z: T) -> Self
    where
        T: Into<Rational>,
    {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Coordinate {
    X(Rational),
    Y(Rational),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
    Join(InputKind),
    Difference(InputKind),
    Split {
        boundary_id: BoundaryId,
        coordinate: Coordinate,
    },
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputKind {
    LinearPolygon(Vec<RationalPoint>),
    Circle {
        center: RationalPoint,
        diameter: Rational,
    },
    Ellipse {
        center: RationalPoint,
        width: Rational,
        height: Rational,
    },
    Cube {
        center: RationalPoint3,
        side_length: Rational,
    },
    Cuboid {
        center: RationalPoint3,
        width: Rational,
        height: Rational,
        depth: Rational,
    },
    Cone {
        center: RationalPoint3,
        radius: Rational,
        height: Rational,
    },
    Sphere {
        center: RationalPoint3,
        radius: Rational,
    },
    Cylinder {
        center: RationalPoint3,
        radius: Rational,
        height: Rational,
    },
}

fn orient_clockwise(mut polygon: UniquePtr<cgal_sys::Polygon>) -> UniquePtr<cgal_sys::Polygon> {
    let _lock = crate::lock();
    use cgal_sys::Orientation;
    if polygon.orientation() != Orientation::COUNTERCLOCKWISE {
        polygon.pin_mut().reverse_orientation();
    }
    polygon
}

impl TryInto<UniquePtr<cgal_sys::Polygon>> for &InputKind {
    type Error = String;

    fn try_into(self) -> Result<UniquePtr<cgal_sys::Polygon>, Self::Error> {
        let _lock = crate::lock();
        match self {
            InputKind::LinearPolygon(vertices) => InputKind::linear(vertices),
            InputKind::Circle { center, diameter } => {
                InputKind::ellipse(center, diameter, diameter)
            }
            InputKind::Ellipse {
                center,
                width,
                height,
            } => InputKind::ellipse(center, width, height),
            InputKind::Cube {
                center,
                side_length,
            } => InputKind::rectangle(&RationalPoint::new(center.x.clone(), center.y.clone()), side_length, side_length), // Note: Temporarily mapped to 2D for backward compatibility in Phase 1
            InputKind::Cuboid {
                center,
                width,
                height,
                ..
            } => InputKind::rectangle(&RationalPoint::new(center.x.clone(), center.y.clone()), width, height),
            InputKind::Cone { center, radius, .. }
            | InputKind::Cylinder { center, radius, .. }
            | InputKind::Sphere { center, radius } => {
                InputKind::ellipse(
                    &RationalPoint::new(center.x.clone(), center.y.clone()),
                    radius,
                    radius,
                )
            }
        }
        .map(orient_clockwise)
    }
}

impl InputKind {
    fn push_curve(
        mut polygon: UniquePtr<cgal_sys::Polygon>,
        curve: &XMonotoneCurve,
    ) -> Result<UniquePtr<Polygon>, String> {
        polygon
            .pin_mut()
            .push_back(curve)
            .map(|()| polygon)
            .map_err(|err| err.to_string())
    }

    fn linear(vertices: &[RationalPoint]) -> Result<UniquePtr<cgal_sys::Polygon>, String> {
        let _lock = crate::lock();
        if vertices.len() < 3 {
            Err(String::from("Vertices should have at least 3 points"))
        } else {
            vertices
                .iter()
                .zip(vertices.iter().cycle().skip(1))
                .take(vertices.len())
                .map(|(source, target)| {
                    cgal_sys::construct_linear_curve(&Point::from(source), &Point::from(target))
                        .map_err(|err| err.to_string())
                })
                .try_fold(cgal_sys::create_polygon(), |polygon, curve_res| {
                    curve_res.and_then(|curve| Self::push_curve(polygon, &curve))
                })
                .map(Into::into)
        }
    }

    fn ellipse(
        center: &RationalPoint,
        width: &Rational,
        height: &Rational,
    ) -> Result<UniquePtr<cgal_sys::Polygon>, String> {
        let _lock = crate::lock();
        cgal_sys::construct_conic_curve(&center.x, &center.y, width, height)
            .and_then(|curve| cgal_sys::split_conic_curve(&curve))
            .map_err(|err| err.to_string())
            .and_then(|curves| {
                curves
                    .into_iter()
                    .try_fold(cgal_sys::create_polygon(), Self::push_curve)
                    .map(Into::into)
            })
    }

    fn rectangle(
        center: &RationalPoint,
        width: &Rational,
        height: &Rational,
    ) -> Result<UniquePtr<cgal_sys::Polygon>, String> {
        let cx = f64::from(&Algebraic::from(&center.x));
        let cy = f64::from(&Algebraic::from(&center.y));
        let w = f64::from(&Algebraic::from(width));
        let h = f64::from(&Algebraic::from(height));

        let w2 = w / 2.0;
        let h2 = h / 2.0;

        let tl = RationalPoint::new(Rational::try_from(cx - w2)?, Rational::try_from(cy + h2)?);
        let tr = RationalPoint::new(Rational::try_from(cx + w2)?, Rational::try_from(cy + h2)?);
        let br = RationalPoint::new(Rational::try_from(cx + w2)?, Rational::try_from(cy - h2)?);
        let bl = RationalPoint::new(Rational::try_from(cx - w2)?, Rational::try_from(cy - h2)?);
        Self::linear(&[bl, br, tr, tl])
    }
}

pub struct PolygonSet {
    inner: UniquePtr<cgal_sys::PolygonSet>,
    polygon_with_holes: Box<[PolygonWithHoles]>,
}

impl Default for PolygonSet {
    fn default() -> Self {
        let _lock = crate::lock();
        Self {
            inner: cgal_sys::create_polygon_set(),
            polygon_with_holes: Box::default(),
        }
    }
}

impl Clone for PolygonSet {
    fn clone(&self) -> Self {
        let _lock = crate::lock();
        Self {
            inner: cgal_sys::clone_polygon_set(&self.inner),
            polygon_with_holes: self.polygon_with_holes.clone(),
        }
    }
}

impl Debug for PolygonSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PolygonSet")
            .field("polygon_with_holes", &self.polygon_with_holes)
            .finish()
    }
}

impl PolygonSet {
    pub fn from_inputs(inputs: &[Input]) -> Result<Self, String> {
        let _lock = crate::lock();
        inputs
            .iter()
            .try_fold(PolygonSet::default(), |mut set, input| {
                set.process_input(input).map(|()| set)
            })
    }

    fn join(&mut self, kind: &InputKind) -> Result<(), String> {
        let polygon: UniquePtr<cgal_sys::Polygon> = kind.try_into()?;
        let polygon_set = self.inner.pin_mut();
        let result = if polygon_set.is_empty() {
            polygon_set.insert(&polygon)
        } else {
            polygon_set.join(&polygon)
        };
        result.map_err(|err| err.to_string())
    }

    fn difference(&mut self, kind: &InputKind) -> Result<(), String> {
        let polygon_set = self.inner.pin_mut();
        if polygon_set.is_empty() {
            Ok(())
        } else {
            let polygon: UniquePtr<cgal_sys::Polygon> = kind.try_into()?;
            polygon_set
                .difference(&polygon)
                .map_err(|err| err.to_string())
        }
    }

    fn split_boundary(
        &mut self,
        boundary_id: &BoundaryId,
        coordinate: &Coordinate,
    ) -> Result<(), String> {
        let curve = self
            .polygon_with_holes()
            .iter()
            .find_map(|polygon| {
                polygon
                    .boundaries_iter()
                    .find_map(|(id, curve)| (id == *boundary_id).then_some(curve))
            })
            .cloned()
            .ok_or_else(|| format!("Boundary id {boundary_id:#?} is invalid"))?;
        let point = match (&curve, coordinate) {
            (Curve::Line(LineSegment::Horizontal(line)), Coordinate::X(x)) => {
                let x = line.clamp_x(&Algebraic::from(x));
                line.point_at_x(&x)
            }
            (Curve::Line(LineSegment::Vertical(line)), Coordinate::Y(y)) => {
                let y = line.clamp_y(&Algebraic::from(y));
                line.point_at_y(&y)
            }
            (Curve::Line(LineSegment::Oblique(line)), Coordinate::X(x)) => {
                let x = line.clamp_x(&Algebraic::from(x));
                line.point_at_x(&x)
            }
            (Curve::Ellipse(arc), Coordinate::X(x)) => {
                let x = arc.clamp_x(&Algebraic::from(x));
                arc.point_at_x(&x)
            }
            (curve, coordinate) => {
                return Err(format!("Splitting {curve:#?} at {coordinate:#?} is not supported"));
            }
        }?;
        if curve.end_points().start() == &point || curve.end_points().end() == &point {
            return Err(String::from("Cannot split a curve at one of it's endpoint"));
        }
        cgal_sys::split_curve(self.inner.pin_mut(), curve.borrow(), &point)
            .map_err(|err| err.to_string())
    }

    pub fn process_input(&mut self, input: &Input) -> Result<(), String> {
        let _lock = crate::lock();
        let result = match input {
            Input::Join(kind) => self.join(kind),
            Input::Difference(kind) => self.difference(kind),
            Input::Split {
                boundary_id,
                coordinate,
            } => self.split_boundary(boundary_id, coordinate),
        };
        if result.is_ok() {
            self.polygon_with_holes = cgal_sys::polygon_with_holes(&self.inner)
                .iter()
                .map(cgal_sys::clone_polygon_with_holes)
                .map(Into::into)
                .collect();
        }
        result
    }

    pub fn polygon_with_holes(&self) -> &[PolygonWithHoles] {
        &self.polygon_with_holes
    }

    pub fn curves(&self) -> impl Iterator<Item = &Curve> {
        self.polygon_with_holes
            .iter()
            .flat_map(PolygonWithHoles::boundaries_iter)
            .map(|(_, curve)| curve)
    }

    pub fn vertices(&self) -> impl Iterator<Item = &Point> {
        self.curves().map(|curve| curve.end_points().end())
    }

    pub fn is_empty(&self) -> bool {
        let _lock = crate::lock();
        self.inner.is_empty()
    }
    
    pub fn clear(&mut self) {
        let _lock = crate::lock();
        self.inner.pin_mut().clear();
    }
}

impl Drop for PolygonSet {
    fn drop(&mut self) {
        let _lock = crate::lock();
        let mut inner = UniquePtr::null();
        std::mem::swap(&mut self.inner, &mut inner);
        drop(inner);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        curve::{Curve, EndPoints},
        polygon_with_holes::{CurveId, HoleId},
    };

    #[test]
    fn default_gives_empty() {
        assert!(PolygonSet::default().is_empty());
    }

    #[test]
    fn join_works() {
        let mut set = PolygonSet::default();

        let input = Input::Join(InputKind::LinearPolygon(vec![
            RationalPoint::default(),
            RationalPoint::new(1, 0),
            RationalPoint::new(1, 1),
        ]));
        set.process_input(&input).unwrap();
        assert!(!set.is_empty());
        assert_eq!(set.polygon_with_holes().len(), 1);

        set.clear();
        assert!(set.is_empty());

        let input = Input::Join(InputKind::Circle {
            center: RationalPoint::default(),
            diameter: Rational::from(100),
        });
        set.process_input(&input).unwrap();
        assert!(!set.is_empty());
        assert_eq!(set.polygon_with_holes().len(), 1);
    }

    #[test]
    fn union_of_overlapping_circle_and_square_works() {
        let mut set = PolygonSet::default();
        assert!(set
            .process_input(&Input::Join(InputKind::LinearPolygon(vec![
                RationalPoint::default(),
                RationalPoint::new(1, 0),
                RationalPoint::new(1, 1),
                RationalPoint::new(0, 1),
            ])))
            .is_ok());
        assert!(set
            .process_input(&Input::Join(InputKind::Circle {
                center: RationalPoint::new(1, 1),
                diameter: Rational::new_fraction_unwrapped(1, 5),
            }))
            .is_ok());
    }

    #[test]
    fn trapezium_addition_works() {
        let mut set = PolygonSet::default();
        assert!(set
            .process_input(&Input::Join(InputKind::LinearPolygon(vec![
                RationalPoint::default(),
                RationalPoint::new(0, 2),
                RationalPoint::new(2, 2),
                RationalPoint::new(2, -2),
            ])))
            .is_ok());
    }

    #[test]
    fn difference_works() {
        let mut set = PolygonSet::default();
        let input_kind = InputKind::LinearPolygon(vec![
            RationalPoint::default(),
            RationalPoint::new(1, 0),
            RationalPoint::new(1, 1),
        ]);
        set.process_input(&Input::Difference(input_kind.clone()))
            .unwrap();
        assert!(set.is_empty());
        assert!(set.polygon_with_holes.is_empty());
        set.process_input(&Input::Join(input_kind.clone())).unwrap();
        assert!(!set.is_empty());
        assert_eq!(set.polygon_with_holes().len(), 1);
        set.process_input(&Input::Difference(input_kind)).unwrap();
        assert!(set.is_empty());
        assert!(set.polygon_with_holes.is_empty());
    }

    #[test]
    fn split_works() {
        let mut set = PolygonSet::default();
        set.process_input(&Input::Join(InputKind::LinearPolygon(vec![
            RationalPoint::default(),
            RationalPoint::new(1, 0),
            RationalPoint::new(1, 1),
            RationalPoint::new(0, 1),
        ])))
        .unwrap();
        set.process_input(&Input::Difference(InputKind::Circle {
            center: RationalPoint::new(
                Rational::new_fraction_unwrapped(1, 2),
                Rational::new_fraction_unwrapped(1, 2),
            ),
            diameter: Rational::new_fraction_unwrapped(3, 5),
        }))
        .unwrap();
        let upper_arc_end_points = {
            let start = Point::new(
                Rational::new_fraction_unwrapped(1, 5),
                Rational::new_fraction_unwrapped(1, 2),
            );
            let end = Point::new(
                Rational::new_fraction_unwrapped(4, 5),
                Rational::new_fraction_unwrapped(1, 2),
            );
            EndPoints::new(start.clone(), end.clone())
        };

        let upper_arc_id = BoundaryId::Hole(HoleId(0), CurveId(0));
        let upper_arc = set.polygon_with_holes()[0].boundary_with_id(&upper_arc_id);
        let Curve::Ellipse(upper_arc) = upper_arc else {
            panic!("Hole upper arc should be elliptical")
        };

        let x = Rational::new_fraction_unwrapped(1, 2);
        let point = upper_arc.point_at_x(&Algebraic::from(&x)).unwrap();
        let coordinate = Coordinate::X(x);

        set.process_input(&Input::Split {
            boundary_id: upper_arc_id,
            coordinate: coordinate.clone(),
        })
        .unwrap();

        let mut arc_end_points = Vec::with_capacity(3);
        arc_end_points.push(EndPoints::new(
            point.clone(),
            upper_arc_end_points.end().clone(),
        ));
        arc_end_points.push(EndPoints::new(
            arc_end_points[0].end().clone(),
            upper_arc_end_points.start().clone(),
        ));
        arc_end_points.push(EndPoints::new(
            arc_end_points[1].end().clone(),
            arc_end_points[0].start().clone(),
        ));

        set.curves()
            .filter_map(|curve| match curve {
                Curve::Line(_) => None,
                Curve::Ellipse(arc) => Some(&arc.end_points),
            })
            .zip(arc_end_points)
            .for_each(|(lhs, rhs)| {
                approx::assert_abs_diff_eq!(lhs, &rhs);
            });
    }

    #[test]
    fn clear_works() {
        let mut set = PolygonSet::default();
        let input = Input::Join(InputKind::LinearPolygon(vec![
            RationalPoint::default(),
            RationalPoint::new(1, 0),
            RationalPoint::new(1, 1),
        ]));
        set.process_input(&input).unwrap();
        assert_eq!(set.polygon_with_holes().len(), 1);
        set.clear();
        assert!(set.is_empty());
    }
}
