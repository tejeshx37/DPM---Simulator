use crate::num::Rational;
use crate::polygon_set::RationalPoint3;
use cxx::UniquePtr;
use std::fmt::Debug;

pub struct PolyhedronSet {
    inner: UniquePtr<cgal_sys::PolyhedronSet3>,
}

impl Default for PolyhedronSet {
    fn default() -> Self {
        let _lock = crate::lock();
        Self {
            inner: cgal_sys::create_polyhedron_set(),
        }
    }
}

impl Clone for PolyhedronSet {
    fn clone(&self) -> Self {
        let _lock = crate::lock();
        Self {
            inner: cgal_sys::clone_polyhedron_set(&self.inner),
        }
    }
}

impl Debug for PolyhedronSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PolyhedronSet").finish()
    }
}

impl PolyhedronSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        let _lock = crate::lock();
        self.inner.is_empty()
    }

    pub fn join(&mut self, other: &Self) {
        let _lock = crate::lock();
        self.inner.pin_mut().join(&other.inner);
    }

    pub fn difference(&mut self, other: &Self) {
        let _lock = crate::lock();
        self.inner.pin_mut().difference(&other.inner);
    }

    pub fn intersection(&mut self, other: &Self) {
        let _lock = crate::lock();
        self.inner.pin_mut().intersection(&other.inner);
    }

    pub fn get_vertices(&self) -> Vec<RationalPoint3> {
        self.get_mesh_rational().0
    }

    pub fn get_triangles(&self) -> Vec<u32> {
        self.get_mesh().triangles
    }


    pub fn get_mesh(&self) -> cgal_sys::Mesh3D {
        let _lock = crate::lock();
        self.inner.get_mesh()
    }

    pub fn get_mesh_rational(&self) -> (Vec<RationalPoint3>, Vec<u32>) {
        let _lock = crate::lock();
        let mesh = self.inner.get_mesh();
        let vertices = mesh.vertices.iter()
            .map(|p| {
                RationalPoint3::new(
                    Rational::try_from(p.x).unwrap(),
                    Rational::try_from(p.y).unwrap(),
                    Rational::try_from(p.z).unwrap(),
                )
            })
            .collect();
        (vertices, mesh.triangles)
    }

}

impl PolyhedronSet {
    pub fn from_inputs(inputs: &[crate::polygon_set::Input]) -> Result<Self, String> {
        let _lock = crate::lock();
        inputs
            .iter()
            .try_fold(PolyhedronSet::default(), |mut set, input| {
                set.process_input(input).map(|()| set)
            })
    }

    fn process_input(&mut self, input: &crate::polygon_set::Input) -> Result<(), String> {
        match input {
            crate::polygon_set::Input::Join(kind) => self.join_input(kind),
            crate::polygon_set::Input::Difference(kind) => self.difference_input(kind),
            crate::polygon_set::Input::Split { .. } => Ok(()), // Ignore splits for 3D currently
        }
    }

    fn join_input(&mut self, kind: &crate::polygon_set::InputKind) -> Result<(), String> {
        let other = Self::from_kind(kind)?;
        if let Some(other) = other {
            if self.is_empty() {
                self.inner = cgal_sys::clone_polyhedron_set(&other.inner);
            } else {
                self.join(&other);
            }
        }
        Ok(())
    }

    fn difference_input(&mut self, kind: &crate::polygon_set::InputKind) -> Result<(), String> {
        if self.is_empty() {
            return Ok(());
        }
        let other = Self::from_kind(kind)?;
        if let Some(other) = other {
            self.difference(&other);
        }
        Ok(())
    }

    fn from_kind(kind: &crate::polygon_set::InputKind) -> Result<Option<Self>, String> {
        use crate::polygon_set::InputKind;
        match kind {
            InputKind::Cube { center, side_length } => {
                Ok(Some(Self::create_cube(center, side_length)))
            }
            InputKind::Cuboid { center, width, height, depth } => {
                Ok(Some(Self::create_cuboid(center, width, height, depth)))
            }
            InputKind::Sphere { center, radius } => {
                // Number of latitudes and longitudes can be configurable or derived
                // Use a default reasonable value for now
                let num_lat = 32;
                let num_lon = 32;
                Ok(Some(Self::create_approximated_sphere(center, radius, num_lat, num_lon)))
            }
            InputKind::Cone { center, radius, height } => {
                let num_segments = 32;
                Ok(Some(Self::create_approximated_cone(center, radius, height, num_segments)))
            }
            InputKind::Cylinder { center, radius, height } => {
                let num_segments = 32;
                Ok(Some(Self::create_approximated_cylinder(center, radius, height, num_segments)))
            }
            InputKind::LinearPolygon(vertices) => {
                let default_height = crate::num::Rational::from(1);
                Ok(Some(Self::create_extruded_polygon(vertices, &default_height)))
            }
            InputKind::Circle { center, diameter } => {
                let default_height = crate::num::Rational::from(1);
                let radius_f64 = diameter.to_f64() / 2.0;
                let radius = crate::num::Rational::try_from(radius_f64).unwrap();
                let center_3d = crate::polygon_set::RationalPoint3::new(center.x.clone(), center.y.clone(), crate::num::Rational::from(0));
                let num_segments = 32;
                Ok(Some(Self::create_approximated_cylinder(&center_3d, &radius, &default_height, num_segments)))
            }
            _ => Ok(None), // Other 2D kinds (Ellipse) are ignored in 3D evaluation for now
        }
    }

    pub fn create_extruded_polygon(
        vertices: &[crate::polygon_set::RationalPoint],
        height: &crate::num::Rational,
    ) -> Self {
        let pts_x: Vec<f64> = vertices.iter().map(|p| p.x.to_f64()).collect();
        let pts_y: Vec<f64> = vertices.iter().map(|p| p.y.to_f64()).collect();
        let inner =
            cgal_sys::create_extruded_polygon(&pts_x, &pts_y, height);
        Self { inner }
    }
}

impl PolyhedronSet {
    pub fn create_cube(center: &RationalPoint3, size: &Rational) -> Self {
        let _lock = crate::lock();
        Self {
            inner: cgal_sys::create_cube(&center.x, &center.y, &center.z, size),
        }
    }

    pub fn create_cuboid(
        center: &RationalPoint3,
        width: &Rational,
        height: &Rational,
        depth: &Rational,
    ) -> Self {
        let _lock = crate::lock();
        Self {
            inner: cgal_sys::create_cuboid(&center.x, &center.y, &center.z, width, height, depth),
        }
    }

    pub fn create_approximated_sphere(
        center: &RationalPoint3,
        radius: &Rational,
        num_lat: u32,
        num_lon: u32,
    ) -> Self {
        let _lock = crate::lock();
        Self {
            inner: cgal_sys::create_approximated_sphere(
                &center.x, &center.y, &center.z, radius, num_lat, num_lon,
            ),
        }
    }

    pub fn create_approximated_cone(
        center: &RationalPoint3,
        radius: &Rational,
        height: &Rational,
        num_segments: u32,
    ) -> Self {
        let _lock = crate::lock();
        Self {
            inner: cgal_sys::create_approximated_cone(
                &center.x, &center.y, &center.z, radius, height, num_segments,
            ),
        }
    }

    pub fn create_approximated_cylinder(
        center: &RationalPoint3,
        radius: &Rational,
        height: &Rational,
        num_segments: u32,
    ) -> Self {
        let _lock = crate::lock();
        Self {
            inner: cgal_sys::create_approximated_cylinder(
                &center.x, &center.y, &center.z, radius, height, num_segments,
            ),
        }
    }
}
