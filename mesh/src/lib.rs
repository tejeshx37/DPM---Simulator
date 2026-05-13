use cgal::{curve::Curve, triangulation, BoundaryId, PolygonWithHoles};
use derive_getters::Getters;
use fxhash::{FxHashMap, FxHashSet};
use nalgebra::Vector3;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use rayon::prelude::*;
use std::iter;

const ASPECT_BOUND: f64 = 0.125;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeedingPattern {
    Grid,
    Hexagonal,
    Fibonacci,
    Random,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum Region {
    BoundingBox { min: Vector3<f64>, max: Vector3<f64> },
    PolygonZone { points: Vec<[f64; 2]> },
    SDF { center: Vector3<f64>, radius: f64 },
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct SeedingStrategy {
    pub pattern: SeedingPattern,
    pub density: f64, // points per unit volume
    pub radius: f64,  // individual particle radius (if applicable)
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct SeedingRegion {
    pub region: Region,
    pub strategy: SeedingStrategy,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct SeedingConfig {
    pub regions: Vec<SeedingRegion>,
    pub default_strategy: Option<SeedingStrategy>,
}

impl Region {
    pub fn contains(&self, p: &Vector3<f64>) -> bool {
        match self {
            Region::BoundingBox { min, max } => {
                p.x >= min.x && p.x <= max.x &&
                p.y >= min.y && p.y <= max.y &&
                p.z >= min.z && p.z <= max.z
            }
            Region::PolygonZone { points } => {
                is_inside_point_list(p, points)
            }
            Region::SDF { center, radius } => {
                (p - center).norm_squared() <= radius * radius
            }
        }
    }

    pub fn bounds(&self) -> (Vector3<f64>, Vector3<f64>) {
        match self {
            Region::BoundingBox { min, max } => (*min, *max),
            Region::PolygonZone { points } => {
                let mut min_x = f64::MAX;
                let mut max_x = f64::MIN;
                let mut min_y = f64::MAX;
                let mut max_y = f64::MIN;
                for p in points {
                    min_x = min_x.min(p[0]);
                    max_x = max_x.max(p[0]);
                    min_y = min_y.min(p[1]);
                    max_y = max_y.max(p[1]);
                }
                (Vector3::new(min_x, min_y, 0.0), Vector3::new(max_x, max_y, f64::MAX))
            }
            Region::SDF { center, radius } => {
                let r_vec = Vector3::new(*radius, *radius, *radius);
                (center - r_vec, center + r_vec)
            }
        }
    }
}

impl SeedingStrategy {
    pub fn generate_points(&self, bounds_min: Vector3<f64>, bounds_max: Vector3<f64>) -> Vec<Vector3<f64>> {
        let mut points = Vec::new();
        let dx = bounds_max.x - bounds_min.x;
        let dy = bounds_max.y - bounds_min.y;
        let dz = (bounds_max.z - bounds_min.z).max(1e-6);
        let volume = dx * dy * dz;
        let target_count = (volume * self.density).ceil() as usize;

        if target_count == 0 { return points; }

        match self.pattern {
            SeedingPattern::Grid => {
                let h = (1.0 / self.density).powf(1.0/3.0);
                let nx = (dx / h).ceil() as usize;
                let ny = (dy / h).ceil() as usize;
                let nz = (dz / h).ceil() as usize;
                for ix in 0..=nx {
                    for iy in 0..=ny {
                        for iz in 0..=nz {
                            points.push(Vector3::new(
                                bounds_min.x + (ix as f64 / (nx as f64).max(1.0)) * dx,
                                bounds_min.y + (iy as f64 / (ny as f64).max(1.0)) * dy,
                                bounds_min.z + (iz as f64 / (nz as f64).max(1.0)) * dz,
                            ));
                        }
                    }
                }
            }
            SeedingPattern::Hexagonal => {
                let d = (std::f64::consts::SQRT_2 / self.density).powf(1.0/3.0);
                let dx_step = d;
                let dy_step = d * 3.0f64.sqrt() / 2.0;
                let dz_step = d * (2.0f64 / 3.0f64).sqrt();

                let nx = (dx / dx_step).ceil() as usize;
                let ny = (dy / dy_step).ceil() as usize;
                let nz = (dz / dz_step).ceil() as usize;

                for iz in 0..=nz {
                    let z = bounds_min.z + iz as f64 * dz_step;
                    for iy in 0..=ny {
                        let y = bounds_min.y + iy as f64 * dy_step;
                        for ix in 0..=nx {
                            let mut x = bounds_min.x + ix as f64 * dx_step;
                            if iy % 2 == 1 { x += dx_step / 2.0; }
                            if iz % 2 == 1 { x += dx_step / 2.0; }
                            points.push(Vector3::new(x, y, z));
                        }
                    }
                }
            }
            SeedingPattern::Fibonacci => {
                let golden_ratio = (1.0 + 5.0f64.sqrt()) / 2.0;
                for i in 0..target_count {
                    let x = (i as f64 / golden_ratio) % 1.0;
                    let y = (i as f64 / golden_ratio.powi(2)) % 1.0;
                    let z = i as f64 / target_count as f64;
                    points.push(Vector3::new(
                        bounds_min.x + x * dx,
                        bounds_min.y + y * dy,
                        bounds_min.z + z * dz,
                    ));
                }
            }
            SeedingPattern::Random => {
                let mut rng = Pcg64::seed_from_u64(42);
                for _ in 0..target_count {
                    points.push(Vector3::new(
                        bounds_min.x + rng.gen::<f64>() * dx,
                        bounds_min.y + rng.gen::<f64>() * dy,
                        bounds_min.z + rng.gen::<f64>() * dz,
                    ));
                }
            }
        }
        points
    }
}

fn is_inside_point_list(p: &Vector3<f64>, points: &[[f64; 2]]) -> bool {
    let mut winding_number = 0;
    let px = p.x;
    let py = p.y;
    for i in 0..points.len() {
        let p1 = &points[i];
        let p2 = &points[(i + 1) % points.len()];
        if p1[1] <= py {
            if p2[1] > py && (p2[0] - p1[0]) * (py - p1[1]) - (px - p1[0]) * (p2[1] - p1[1]) > 0.0 {
                winding_number += 1;
            }
        } else if p2[1] <= py && (p2[0] - p1[0]) * (py - p1[1]) - (px - p1[0]) * (p2[1] - p1[1]) < 0.0 {
            winding_number -= 1;
        }
    }
    winding_number != 0
}

pub type PointIdxToIdsMap = FxHashMap<usize, FxHashSet<BoundaryId>>;
pub type BoundaryIdToCountMap = FxHashMap<BoundaryId, FxHashSet<usize>>;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum Constraint {
    Line([Vector3<f64>; 2]),
    PolyLine(Box<[[Vector3<f64>; 2]]>),
}

impl Constraint {
    fn create(curve: &Curve, num_points: f64, total_perimeter: f64) -> Self {
        match curve {
            Curve::Line(line) => {
                let start = line.end_points().start();
                let end = line.end_points().end();
                Constraint::Line([
                    Vector3::new(start.x().double_value(), start.y().double_value(), 0.0),
                    Vector3::new(end.x().double_value(), end.y().double_value(), 0.0),
                ])
            }
            Curve::Ellipse(_) => {
                let split_count = (curve.length() * num_points) / total_perimeter;
                let generated_points = curve.split(split_count as u32);
                Constraint::PolyLine(
                    generated_points
                        .iter()
                        .map(|p| Vector3::new(p.x().double_value(), p.y().double_value(), 0.0))
                        .zip(generated_points.iter().skip(1).map(|p| Vector3::new(p.x().double_value(), p.y().double_value(), 0.0)))
                        .map(|(a, b)| [a, b])
                        .collect(),
                )
            }
        }
    }

    fn contains_point(&self, q: &Vector3<f64>) -> bool {
        match self {
            Constraint::Line(arr) => is_on_same_segment(&arr[0], q, &arr[1]),
            Constraint::PolyLine(boxed) => {
                boxed.iter().any(|[p, r]| is_on_same_segment(p, q, r))
            }
        }
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &[Vector3<f64>; 2]> + '_> {
        match self {
            Constraint::Line(arr) => Box::new(iter::once(arr)),
            Constraint::PolyLine(boxed) => Box::new(boxed.iter()),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Getters)]
pub struct Mesh {
    triangulation_data: triangulation::Data,
    constraints: Box<[(BoundaryId, Constraint)]>,
    point_id_map: PointIdxToIdsMap,
    boundary_point_map: BoundaryIdToCountMap,
    smallest_side_length: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Init,
    GeneratingConstraints,
    Triangulating,
    GeneratingAssociativeData,
    FindingSmallestEdge,
    Done,
}

#[derive(Default)]
pub enum Callback<'a> {
    Some(Box<dyn FnMut(State) + 'a>),
    #[default]
    None,
}

impl<'a> Callback<'a> {
    fn invoke(&mut self, state: State) {
        match self {
            Callback::Some(f) => f(state),
            Callback::None => {}
        }
    }
}

impl<'a, F> From<F> for Callback<'a>
where
    F: FnMut(State) + 'a,
{
    fn from(value: F) -> Self {
        Self::Some(Box::new(value))
    }
}

impl Mesh {
    pub fn generate_from_polyhedron(
        polyhedron: &cgal::PolyhedronSet,
        num_points: u32,
        size_bound_override: Option<f64>,
        mut state_callback: Callback,
    ) -> Result<Self, String> {
        state_callback.invoke(State::Init);

        let vertices = polyhedron.get_vertices();
        if vertices.is_empty() {
            return Err(String::from("Polyhedron is empty. Cannot generate mesh."));
        }

        state_callback.invoke(State::GeneratingConstraints);

        // Approximate total size for bounding
        let triangles = polyhedron.get_triangles();
        let mut total_perimeter = 0.0;
        let to_f64 = |r: &cgal::num::Rational| f64::from(cgal::num::Algebraic::from(r));
        
        for t in triangles.chunks_exact(3) {
            let p1 = &vertices[t[0] as usize];
            let p2 = &vertices[t[1] as usize];
            let p3 = &vertices[t[2] as usize];
            let v1 = Vector3::new(to_f64(&p1.x), to_f64(&p1.y), to_f64(&p1.z));
            let v2 = Vector3::new(to_f64(&p2.x), to_f64(&p2.y), to_f64(&p2.z));
            let v3 = Vector3::new(to_f64(&p3.x), to_f64(&p3.y), to_f64(&p3.z));
            total_perimeter += (v1 - v2).magnitude() + (v2 - v3).magnitude() + (v3 - v1).magnitude();
        }
        
        let size_bound = size_bound_override.unwrap_or(total_perimeter / (num_points as f64).max(1.0));
        
        let point_cloud: Vec<Vector3<f64>> = vertices
            .into_iter()
            .map(|p| Vector3::new(to_f64(&p.x), to_f64(&p.y), to_f64(&p.z)))
            .collect();

        Self::triangulate_and_build(point_cloud, size_bound, state_callback)
    }

    pub fn generate(
        polygon: &PolygonWithHoles,
        num_points: u32,
        size_bound_override: Option<f64>,
        thickness: f64,
        primitive: Option<&cgal::PolygonSetInputKind>,
        seeding_config: Option<SeedingConfig>,
        mut state_callback: Callback,
    ) -> Result<Self, String> {
        state_callback.invoke(State::Init);

        if let Some(primitive) = primitive {
            match primitive {
                cgal::PolygonSetInputKind::Sphere { center, radius } => {
                    let r = f64::from(&cgal::num::Algebraic::from(radius));
                    return Self::generate_sphere(center, r, num_points, size_bound_override, state_callback);
                }
                cgal::PolygonSetInputKind::Cone { center, radius, height } => {
                    let r = f64::from(&cgal::num::Algebraic::from(radius));
                    let h = f64::from(&cgal::num::Algebraic::from(height));
                    return Self::generate_cone(center, r, h, num_points, size_bound_override, state_callback);
                }
                _ => {}
            }
        }

        let num_points_f = num_points as f64;
        let total_perimeter: f64 = polygon
            .boundaries_iter()
            .map(|(_, curve)| curve.length())
            .sum();
        
        if total_perimeter <= 0.0 {
            return Err(String::from("Polygon has zero perimeter. Cannot generate mesh."));
        }

        let size_bound = size_bound_override.unwrap_or(total_perimeter / num_points_f);
        if !size_bound.is_finite() || size_bound <= 0.0 {
             return Err(String::from("Invalid size bound calculated. Check your point count."));
        }

        state_callback.invoke(State::GeneratingConstraints);

        let constraints: Vec<(BoundaryId, Constraint)> = polygon
            .boundaries_iter()
            .map(|(boundary_id, curve)| {
                (
                    boundary_id,
                    Constraint::create(curve, num_points_f, total_perimeter),
                )
            })
            .collect();

        let mut point_cloud: Vec<Vector3<f64>> = constraints
            .iter()
            .flat_map(|(_, constraint)| constraint.iter())
            .flat_map(|line| vec![line[0], line[1]])
            .collect();

        // --- Volumetric Internal Points ---
        if let Some(config) = seeding_config {
            for seeding_region in &config.regions {
                let (mut b_min, mut b_max) = seeding_region.region.bounds();
                // Clamp bounds to thickness
                b_min.z = b_min.z.max(0.0);
                b_max.z = b_max.z.min(thickness);
                
                let points = seeding_region.strategy.generate_points(b_min, b_max);
                for p in points {
                    if seeding_region.region.contains(&p) && is_inside_polygon(&p, polygon) {
                        point_cloud.push(p);
                    }
                }
            }
            if let Some(default_strategy) = config.default_strategy {
                let mut min_x = f64::MAX;
                let mut max_x = f64::MIN;
                let mut min_y = f64::MAX;
                let mut max_y = f64::MIN;
                for p in &point_cloud {
                    min_x = min_x.min(p.x);
                    max_x = max_x.max(p.x);
                    min_y = min_y.min(p.y);
                    max_y = max_y.max(p.y);
                }
                let b_min = Vector3::new(min_x, min_y, 0.0);
                let b_max = Vector3::new(max_x, max_y, thickness);
                let points = default_strategy.generate_points(b_min, b_max);
                for p in points {
                    if is_inside_polygon(&p, polygon) && !config.regions.iter().any(|r| r.region.contains(&p)) {
                        point_cloud.push(p);
                    }
                }
            }
        } else {
            // Default seeding logic
            let mut min_x = f64::MAX;
            let mut max_x = f64::MIN;
            let mut min_y = f64::MAX;
            let mut max_y = f64::MIN;
            for p in &point_cloud {
                min_x = min_x.min(p.x);
                max_x = max_x.max(p.x);
                min_y = min_y.min(p.y);
                max_y = max_y.max(p.y);
            }

            let dx = max_x - min_x;
            let dy = max_y - min_y;
            
            if dx > 0.0 && dy > 0.0 && thickness > 0.0 {
                // Determine grid resolution based on num_points
                let volume = dx * dy * thickness;
                let target_internal = (num_points as f64 * 0.7) as usize; // reserve 70% for internal
                let cell_volume = volume / target_internal as f64;
                let h = cell_volume.powf(1.0/3.0);
                
                let nx = (dx / h).ceil() as usize;
                let ny = (dy / h).ceil() as usize;
                let nz = (thickness / h).ceil() as usize;
                
                for ix in 0..=nx {
                    for iy in 0..=ny {
                        let x = min_x + (ix as f64 / nx as f64) * dx;
                        let y = min_y + (iy as f64 / ny as f64) * dy;
                        let p_check = Vector3::new(x, y, 0.0);
                        
                        if is_inside_polygon(&p_check, polygon) {
                            for iz in 0..=nz {
                                let z = (iz as f64 / nz as f64) * thickness;
                                point_cloud.push(Vector3::new(x, y, z));
                            }
                        }
                    }
                }
            }
        }
            
        // Add extruded boundary points
        let top_boundary_points: Vec<Vector3<f64>> = constraints
            .iter()
            .flat_map(|(_, constraint)| constraint.iter())
            .flat_map(|line| vec![line[0], line[1]])
            .map(|mut p| { p.z = thickness; p })
            .collect();
        point_cloud.extend(top_boundary_points);

        state_callback.invoke(State::Triangulating);

        let triangulation_data =
            triangulation::triangulate(&point_cloud, ASPECT_BOUND, size_bound)?;

        state_callback.invoke(State::GeneratingAssociativeData);

        let (point_id_map, boundary_point_map) =
            generate_associative_data(&triangulation_data, &constraints);

        state_callback.invoke(State::FindingSmallestEdge);

        let smallest_side_length = if triangulation_data.faces().is_empty() {
            0.0
        } else {
            triangulation_data
                .faces()
                .par_iter()
                .map(|face| {
                    let ith_point = |i: usize| triangulation_data.vertices()[i].point();
                    face.0
                        .into_iter()
                        .cycle()
                        .map(ith_point)
                        .zip(face.0.into_iter().skip(1).cycle().map(ith_point))
                        .take(face.0.len())
                        .map(|(p, q)| (p - q).magnitude_squared())
                        .reduce(f32::min)
                        .unwrap_or(f32::MAX)
                })
                .reduce(|| f32::MAX, f32::min)
                .sqrt() as f64
        };

        let constraints = constraints.into_boxed_slice();

        state_callback.invoke(State::Done);

        Ok(Self {
            point_id_map,
            boundary_point_map,
            constraints,
            smallest_side_length,
            triangulation_data,
        })
    }
}

// q = p + l(r - p). l belongs to [0, 1]
fn is_on_same_segment(p: &Vector3<f64>, q: &Vector3<f64>, r: &Vector3<f64>) -> bool {
    let tolerance = 1e-12;
    // Fast Bounding Box check
    if q.x < p.x.min(r.x) - tolerance || q.x > p.x.max(r.x) + tolerance ||
       q.y < p.y.min(r.y) - tolerance || q.y > p.y.max(r.y) + tolerance {
        return false;
    }
    let tolerance = 1e-6;
    let qp = q - p;
    let rp = r - p;

    if qp.norm_squared() <= tolerance * tolerance {
        return true;
    }

    let cross = qp.cross(&rp);
    if cross.norm_squared() > tolerance * tolerance {
        return false;
    }

    let dot = rp.dot(&qp);
    if dot.abs() < tolerance {
        return false;
    }

    let l = qp.dot(&qp) / dot;
    if !l.is_finite() {
        return false;
    }

    l >= -tolerance && l <= 1.0 + tolerance
}

fn is_inside_polygon(p: &Vector3<f64>, polygon: &PolygonWithHoles) -> bool {
    let mut winding_number = 0;
    let px = p.x;
    let py = p.y;

    for (_, curve) in polygon.boundaries_iter() {
        let points = match curve {
            Curve::Line(line) => vec![line.end_points().start().clone(), line.end_points().end().clone()],
            Curve::Ellipse(arc) => {
                let mut pts = Vec::new();
                let start = arc.end_points.start();
                let end = arc.end_points.end();
                pts.push(start.clone());
                pts.push(end.clone());
                pts
            }
        };

        for i in 0..points.len() {
            let p1 = &points[i];
            let p2 = &points[(i + 1) % points.len()];
            
            let p1x = p1.x().double_value();
            let p1y = p1.y().double_value();
            let p2x = p2.x().double_value();
            let p2y = p2.y().double_value();

            if p1y <= py {
                if p2y > py && (p2x - p1x) * (py - p1y) - (px - p1x) * (p2y - p1y) > 0.0 {
                    winding_number += 1;
                }
            } else if p2y <= py && (p2x - p1x) * (py - p1y) - (px - p1x) * (p2y - p1y) < 0.0 {
                winding_number -= 1;
            }
        }
    }
    winding_number != 0
}

fn generate_associative_data(
    data: &triangulation::Data,
    constraints: &[(BoundaryId, Constraint)],
) -> (PointIdxToIdsMap, BoundaryIdToCountMap) {
    let results: Vec<(usize, FxHashSet<BoundaryId>)> = data.vertices()
        .par_iter()
        .enumerate()
        .filter_map(|(index, vertex)| {
            let p = vertex.point().map(|v| v as f64);
            let boundary_ids: FxHashSet<BoundaryId> = constraints
                .iter()
                .filter_map(|(id, constraint)| {
                    if constraint.contains_point(&p) { Some(*id) } else { None }
                })
                .collect();
            if boundary_ids.is_empty() {
                None
            } else {
                Some((index, boundary_ids))
            }
        })
        .collect();

    let mut point_id_map = PointIdxToIdsMap::default();
    let mut boundary_point_map = BoundaryIdToCountMap::default();
    
    for (index, ids) in results {
        for id in &ids {
            boundary_point_map.entry(*id).or_default().insert(index);
        }
        point_id_map.insert(index, ids);
    }
    (point_id_map, boundary_point_map)
}

impl Mesh {
    fn generate_sphere(
        center: &cgal::RationalPoint3,
        radius: f64,
        num_points: u32,
        size_bound_override: Option<f64>,
        mut state_callback: Callback,
    ) -> Result<Self, String> {
        state_callback.invoke(State::GeneratingConstraints);
        let cx = f64::from(&cgal::num::Algebraic::from(&center.x));
        let cy = f64::from(&cgal::num::Algebraic::from(&center.y));
        let center = Vector3::new(cx, cy, radius); // Center in 3D (z=radius)
        
        let mut point_cloud = Vec::with_capacity(num_points as usize);
        let n = num_points as f64;
        let phi = std::f64::consts::PI * (5.0f64.sqrt() - 1.0); // golden angle in radians

        // Determine number of concentric shells based on total points
        // We want roughly uniform point density throughout the volume
        let num_shells = (n.powf(1.0 / 3.0)) as u32;
        let num_shells = num_shells.max(1);
        let points_per_shell = num_points / num_shells;

        for shell in 1..=num_shells {
            let r_shell = radius * (shell as f64 / num_shells as f64);
            let n_shell = if shell == num_shells {
                num_points - (num_shells - 1) * points_per_shell
            } else {
                points_per_shell
            };

            for i in 0..n_shell {
                let y = 1.0 - (i as f64 / (n_shell as f64 - 1.0).max(1.0)) * 2.0; // y goes from 1 to -1
                let r = (1.0 - y * y).max(0.0).sqrt(); // radius at y

                let theta = phi * i as f64; // golden angle increment

                let x = theta.cos() * r;
                let z = theta.sin() * r;

                point_cloud.push(center + Vector3::new(x * r_shell, y * r_shell, z * r_shell));
            }
        }

        let size_bound = size_bound_override.unwrap_or(4.0 * std::f64::consts::PI * radius * radius / (num_points as f64));
        Self::triangulate_and_build(point_cloud, size_bound, state_callback)
    }

    fn generate_cone(
        center: &cgal::RationalPoint3,
        radius: f64,
        height: f64,
        num_points: u32,
        size_bound_override: Option<f64>,
        mut state_callback: Callback,
    ) -> Result<Self, String> {
        state_callback.invoke(State::GeneratingConstraints);
        let cx = f64::from(&cgal::num::Algebraic::from(&center.x));
        let cy = f64::from(&cgal::num::Algebraic::from(&center.y));
        let center_2d = Vector3::new(cx, cy, 0.0);
        
        let mut point_cloud = Vec::with_capacity(num_points as usize);
        
        // Point at the tip
        point_cloud.push(center_2d + Vector3::new(0.0, 0.0, height));
        
        // Points on the base circle and surface
        let num_base_points = (num_points as f64 * 0.5) as u32;
        let num_surface_points = num_points - num_base_points - 1;
        
        for i in 0..num_base_points {
            let angle = 2.0 * std::f64::consts::PI * (i as f64 / num_base_points as f64);
            point_cloud.push(center_2d + Vector3::new(angle.cos() * radius, angle.sin() * radius, 0.0));
        }
        
        // Layers for surface and internal points
        let num_z_layers = (num_surface_points as f64).powf(1.0 / 3.0) as u32;
        let num_z_layers = num_z_layers.max(1);
        let num_r_layers = num_z_layers; // Roughly equal divisions radially and axially
        
        let points_per_z_layer = num_surface_points / num_z_layers;
        
        if num_z_layers > 0 {
            for l in 1..=num_z_layers {
                let h_frac = l as f64 / (num_z_layers as f64 + 1.0);
                let h = height * h_frac;
                let max_r = radius * (1.0 - h_frac);
                
                // For each z layer, create concentric circles of points
                for r_layer in 1..=num_r_layers {
                    let r_frac = r_layer as f64 / num_r_layers as f64;
                    let r = max_r * r_frac;
                    
                    let points_this_ring = (points_per_z_layer / num_r_layers).max(1);
                    for i in 0..points_this_ring {
                        let angle = 2.0 * std::f64::consts::PI * (i as f64 / points_this_ring as f64);
                        point_cloud.push(center_2d + Vector3::new(angle.cos() * r, angle.sin() * r, h));
                    }
                }
            }
        }

        let size_bound = size_bound_override.unwrap_or(radius * 2.0 / 10.0); // Heuristic
        Self::triangulate_and_build(point_cloud, size_bound, state_callback)
    }

    fn triangulate_and_build(
        point_cloud: Vec<Vector3<f64>>,
        size_bound: f64,
        mut state_callback: Callback,
    ) -> Result<Self, String> {
        state_callback.invoke(State::Triangulating);
        let triangulation_data = triangulation::triangulate(&point_cloud, 10.0, size_bound)?;

        state_callback.invoke(State::GeneratingAssociativeData);
        let point_id_map = PointIdxToIdsMap::default();
        let boundary_point_map = BoundaryIdToCountMap::default();

        state_callback.invoke(State::FindingSmallestEdge);
        let smallest_side_length = if triangulation_data.faces().is_empty() {
            0.0
        } else {
            triangulation_data
                .faces()
                .par_iter()
                .map(|face| {
                    let ith_point = |i: usize| triangulation_data.vertices()[i].point();
                    face.0
                        .into_iter()
                        .cycle()
                        .map(ith_point)
                        .zip(face.0.into_iter().skip(1).cycle().map(ith_point))
                        .take(face.0.len())
                        .map(|(p, q)| (p - q).magnitude_squared())
                        .reduce(f32::min)
                        .unwrap_or(f32::MAX)
                })
                .reduce(|| f32::MAX, f32::min)
                .sqrt() as f64
        };

        state_callback.invoke(State::Done);
        Ok(Self {
            triangulation_data,
            constraints: Box::new([]),
            point_id_map,
            boundary_point_map,
            smallest_side_length,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Callback, Mesh, State};
    use cgal::{num::Rational, PolygonSet, PolygonSetInput, PolygonSetInputKind, RationalPoint};
    use nalgebra::Vector3;
    use test_case::test_case;
    #[test_case(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 0.0), Vector3::new(2.0, 2.0, 0.0) => true)]
    #[test_case(Vector3::new(2.0, 2.0, 0.0), Vector3::new(1.0, 1.0, 0.0), Vector3::new(0.0, 0.0, 0.0) => true)]
    #[test_case(Vector3::new(0.0, 0.0, 0.0), Vector3::new(2.0, 2.0, 0.0), Vector3::new(2.0, 2.0, 0.0) => true)]
    #[test_case(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0), Vector3::new(2.0, 2.0, 0.0) => true)]
    #[test_case(Vector3::new(0.0, 0.0, 0.0), Vector3::new(2.0, 2.0, 0.0), Vector3::new(1.0, 1.0, 0.0) => false)]
    #[test_case(Vector3::new(1.0, 1.0, 0.0), Vector3::new(2.0, 2.0, 0.0), Vector3::new(0.0, 0.0, 0.0) => false)]
    #[test_case(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.1, 0.0), Vector3::new(2.0, 2.0, 0.0) => false)]
    #[test_case(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.9, 0.0), Vector3::new(2.0, 2.0, 0.0) => false)]
    #[test_case(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.1, 0.0), Vector3::new(2.0, 2.0, 0.0) => false)]
    #[test_case(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 0.0), Vector3::new(2.0, 1.9, 0.0) => false)]
    fn is_on_same_segment_works_within_err(
        p: Vector3<f64>,
        q: Vector3<f64>,
        r: Vector3<f64>,
    ) -> bool {
        super::is_on_same_segment(&p, &q, &r)
    }

    #[test_case(100)]
    #[test_case(300)]
    #[test_case(500)]
    #[test_case(1000)]
    fn generate_works(num_points: u32) {
        let mut polygon_set = PolygonSet::default();

        let fraction = |num, den| Rational::new_fraction_i32(num, den).unwrap();

        let input = PolygonSetInput::Join(PolygonSetInputKind::LinearPolygon(vec![
            RationalPoint::default(),
            RationalPoint::new(1, 0),
            RationalPoint::new(1, 1),
            RationalPoint::new(0, 1),
        ]));
        polygon_set.process_input(&input).unwrap();

        let input = PolygonSetInput::Difference(PolygonSetInputKind::Circle {
            center: RationalPoint::new(fraction(1, 2), fraction(1, 2)),
            diameter: fraction(3, 5),
        });
        polygon_set.process_input(&input).unwrap();

        let input = PolygonSetInput::Difference(PolygonSetInputKind::Circle {
            center: RationalPoint::new(fraction(3, 20), fraction(1, 2)),
            diameter: fraction(1, 5),
        });
        polygon_set.process_input(&input).unwrap();

        let input = PolygonSetInput::Difference(PolygonSetInputKind::Circle {
            center: RationalPoint::new(fraction(17, 20), fraction(1, 2)),
            diameter: fraction(1, 5),
        });
        polygon_set.process_input(&input).unwrap();

        let mut states = Vec::with_capacity(6);

        assert!(Mesh::generate(
            &polygon_set.polygon_with_holes()[0],
            num_points,
            None,
            1.0,
            None,
            None,
            Callback::from(|state| states.push(state))
        )
        .is_ok());

        let expected_states = vec![
            State::Init,
            State::GeneratingConstraints,
            State::Triangulating,
            State::GeneratingAssociativeData,
            State::FindingSmallestEdge,
            State::Done,
        ];

        assert_eq!(states, expected_states);
    }

    #[test]
    fn test_seeding_patterns() {
        use super::{SeedingPattern, SeedingStrategy};
        let strategy = SeedingStrategy {
            pattern: SeedingPattern::Grid,
            density: 100.0,
            radius: 0.1,
        };
        let points = strategy.generate_points(Vector3::zeros(), Vector3::new(1.0, 1.0, 1.0));
        assert!(points.len() >= 100);

        let strategy_hex = SeedingStrategy {
            pattern: SeedingPattern::Hexagonal,
            density: 100.0,
            radius: 0.1,
        };
        let points_hex = strategy_hex.generate_points(Vector3::zeros(), Vector3::new(1.0, 1.0, 1.0));
        assert!(points_hex.len() > 0);

        let strategy_fib = SeedingStrategy {
            pattern: SeedingPattern::Fibonacci,
            density: 100.0,
            radius: 0.1,
        };
        let points_fib = strategy_fib.generate_points(Vector3::zeros(), Vector3::new(1.0, 1.0, 1.0));
        assert_eq!(points_fib.len(), 100);
    }

    #[test]
    fn test_regions() {
        use super::{Region};
        let box_region = Region::BoundingBox {
            min: Vector3::zeros(),
            max: Vector3::new(1.0, 1.0, 1.0),
        };
        assert!(box_region.contains(&Vector3::new(0.5, 0.5, 0.5)));
        assert!(!box_region.contains(&Vector3::new(1.5, 0.5, 0.5)));

        let sdf_region = Region::SDF {
            center: Vector3::new(0.5, 0.5, 0.5),
            radius: 0.5,
        };
        assert!(sdf_region.contains(&Vector3::new(0.5, 0.5, 0.5)));
        assert!(sdf_region.contains(&Vector3::new(0.7, 0.7, 0.7))); // Dist is sqrt(0.04*3) = 0.346 < 0.5
        assert!(!sdf_region.contains(&Vector3::new(0.8, 0.8, 0.8)));
    }
}
