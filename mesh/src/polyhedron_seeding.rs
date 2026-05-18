//! Volumetric seeding helpers for closed triangle meshes (polyhedron output).

use crate::is_inside_point_list;
use cgal::PolyhedronSet;
use nalgebra::Vector3;

const EPS: f64 = 1e-12;

pub fn polyhedron_vertices_f64(polyhedron: &PolyhedronSet) -> Result<Vec<Vector3<f64>>, String> {
    polyhedron
        .get_vertices()?
        .into_iter()
        .map(|p| {
            Vector3::new(p.x, p.y, p.z)
        })
        .collect::<Vec<_>>()
        .pipe(Ok)
}

pub fn polyhedron_triangle_indices(polyhedron: &PolyhedronSet) -> Result<Vec<[usize; 3]>, String> {
    let tri = polyhedron.get_triangles()?;
    Ok(tri.chunks_exact(3)
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .collect())
}

trait Pipe {
    fn pipe<F, R>(self, f: F) -> R where F: FnOnce(Self) -> R, Self: Sized;
}
impl<T> Pipe for T {
    fn pipe<F, R>(self, f: F) -> R where F: FnOnce(Self) -> R, Self: Sized { f(self) }
}

fn axis_aligned_bounds(verts: &[Vector3<f64>]) -> Option<(Vector3<f64>, Vector3<f64>)> {
    let mut it = verts.iter().copied();
    let first = it.next()?;
    let mut mn = first;
    let mut mx = first;
    for v in it {
        mn.x = mn.x.min(v.x);
        mn.y = mn.y.min(v.y);
        mn.z = mn.z.min(v.z);
        mx.x = mx.x.max(v.x);
        mx.y = mx.y.max(v.y);
        mx.z = mx.z.max(v.z);
    }
    Some((mn, mx))
}

/// Ray (origin, direction) vs triangle; counts a hit when `t > t_min` and inside the triangle.
fn ray_hits_triangle(
    origin: Vector3<f64>,
    dir: Vector3<f64>,
    v0: Vector3<f64>,
    v1: Vector3<f64>,
    v2: Vector3<f64>,
    t_min: f64,
) -> bool {
    let e1 = v1 - v0;
    let e2 = v2 - v0;
    let pvec = dir.cross(&e2);
    let det = e1.dot(&pvec);
    if det.abs() < EPS {
        return false;
    }
    let inv_det = 1.0 / det;
    let tvec = origin - v0;
    let u = tvec.dot(&pvec) * inv_det;
    if !(0.0..=1.0).contains(&u) {
        return false;
    }
    let qvec = tvec.cross(&e1);
    let v = dir.dot(&qvec) * inv_det;
    if v < 0.0 || u + v > 1.0 {
        return false;
    }
    let t = e2.dot(&qvec) * inv_det;
    t > t_min
}

/// Odd-parity point-in-polyhedron for a closed manifold-ish triangle soup.
pub fn point_in_closed_mesh(p: &Vector3<f64>, verts: &[Vector3<f64>], tris: &[[usize; 3]]) -> bool {
    if tris.is_empty() {
        return false;
    }
    // +X ray; offset slightly to reduce vertex/edge degeneracies with grid points.
    let origin = Vector3::new(p.x + 1e-7, p.y + 3e-7, p.z + 2e-7);
    let dir = Vector3::new(1.0, 0.0, 0.0);
    let mut hits = 0u32;
    for t in tris {
        let v0 = verts[t[0]];
        let v1 = verts[t[1]];
        let v2 = verts[t[2]];
        if ray_hits_triangle(origin, dir, v0, v1, v2, 0.0) {
            hits += 1;
        }
    }
    hits % 2 == 1
}

pub fn region_contains_for_volume_seed(
    region: &crate::Region,
    p: &Vector3<f64>,
    z_poly_min: f64,
    z_poly_max: f64,
) -> bool {
    match region {
        crate::Region::PolygonZone { points } => {
            is_inside_point_list(p, points) && p.z >= z_poly_min && p.z <= z_poly_max
        }
        _ => region.contains(p),
    }
}

pub fn append_volumetric_seeding(
    point_cloud: &mut Vec<Vector3<f64>>,
    polyhedron: &PolyhedronSet,
    config: &crate::SeedingConfig,
) -> Result<(), String> {
    let verts = polyhedron_vertices_f64(polyhedron)?;
    let tris = polyhedron_triangle_indices(polyhedron)?;
    if verts.is_empty() || tris.is_empty() {
        return Ok(());
    }
    let Some((solid_min, solid_max)) = axis_aligned_bounds(&verts) else {
        return Ok(());
    };
    let z_lo = solid_min.z;
    let z_hi = solid_max.z;

    for sr in &config.regions {
        let (b_min, b_max) = sr.region.seeding_bounds_3d(z_lo, z_hi);
        let candidates = sr.strategy.generate_points(b_min, b_max);
        for p in candidates {
            if region_contains_for_volume_seed(&sr.region, &p, z_lo, z_hi)
                && point_in_closed_mesh(&p, &verts, &tris)
            {
                point_cloud.push(p);
            }
        }
    }

    if let Some(def) = &config.default_strategy {
        let (b_min, b_max) = (solid_min, solid_max);
        let candidates = def.generate_points(b_min, b_max);
        for p in candidates {
            let in_explicit = config.regions.iter().any(|r| {
                region_contains_for_volume_seed(&r.region, &p, z_lo, z_hi)
            });
            if !in_explicit && point_in_closed_mesh(&p, &verts, &tris) {
                point_cloud.push(p);
            }
        }
    }
    Ok(())
}
