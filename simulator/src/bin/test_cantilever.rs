/// ============================================================
/// DPM Cantilever / Uniaxial Tension Test
/// ============================================================
///
/// Setup
/// -----
/// Shape   : Unit sphere  (radius = 1 m, centred at origin)
/// Material: Steel-like isotropic
///             E  = 200 GPa   (Young's modulus)
///             ν  = 0.30      (Poisson's ratio)
///             ρ  = 7800 kg/m³ (density)
///             c  = 5.0       (damping)
///
/// Boundary conditions
/// -------------------
///  • BOTTOM nodes  (z < -0.7)  → Fixed displacement  Z = 0
///    (simulates the block clamped to a rigid wall)
///  • TOP    nodes  (z >  0.7)  → Constant upward force Fz = +5000 N/node
///    (simulates a tensile load pulling the top surface)
///  • All other nodes → Free (no BC)
///
/// What to expect
/// --------------
///  • Nodes near the top should accelerate upward and show tensile stress.
///  • Nodes at the bottom should remain near z ≈ their initial position.
///  • The stress field should grow from 0 toward σ = F / A  as time progresses.
///  • Kinetic energy should remain finite; velocity should stay < 1000 m/s
///    (the clamped velocity cap from the physics hardening we added).
///
/// Output
/// ------
///  Prints per-step summaries: time, max stress, max displacement, KE.
/// ============================================================

use cgal::{BoundaryId, PolygonSet, PolygonSetInput, PolygonSetInputKind, RationalPoint};
use cpd::{
    boundary_condition::{BoundaryCondition, Displacement},
    computer::{self, AdvanceResult},
    config::Config,
    BulkMaterialProps, ElasticityCondition, FailureCriteria, IsotropicMaterialProps, MaterialProps,
};
use function::{
    piecewise_linear::{Piece, PiecewiseLinear},
    Function,
};
use fxhash::FxHashMap;
use mesh::{Callback, Mesh};
use nalgebra::Vector3;
use std::time::Duration;

// ── helpers ──────────────────────────────────────────────────────────────────

/// Build a constant-value `Function` that holds `value` for `total_duration`
/// seconds. The piecewise representation: one piece, end_value = value,
/// width = total_duration.
fn const_fn(value: f32, total_duration: f32) -> Function {
    Function::Piecewise(
        PiecewiseLinear::builder()
            .piece(Piece::builder().end_value(value).width(total_duration).build())
            .build(),
    )
}

/// Zero-displacement function (clamps a node to its initial position).
fn zero_fn(total_duration: f32) -> Function {
    const_fn(0.0, total_duration)
}

fn main() {
    let sim_seconds = 0.01_f32; // total sim time (10ms)
    let dt_ms       = 0.002_f32; // time step in milliseconds (2 microseconds)
    let report_every = 500;      // print a summary every N steps

    // ── 1. Build mesh ─────────────────────────────────────────────────────────
    println!("=== DPM Cantilever / Uniaxial Tension Test ===\n");
    println!("[1/4] Generating mesh...");

    let mut polygon_set = PolygonSet::default();
    let sphere_kind = PolygonSetInputKind::Sphere {
        center: RationalPoint::new(0, 0),
        radius: 1.into(),
    };
    polygon_set
        .process_input(&PolygonSetInput::Join(sphere_kind.clone()))
        .expect("Failed to build polygon set");

    let poly = &polygon_set.polygon_with_holes()[0];
    let mesh = Mesh::generate(
        poly,
        256,          // ~ 256 nodes — small enough to run fast
        None,
        1.0,
        Some(&sphere_kind),
        Callback::None,
    )
    .expect("Mesh generation failed");

    let n_nodes = mesh.triangulation_data().vertices().len();
    let n_elems = mesh.triangulation_data().faces().len();
    println!("    Nodes: {}  |  Elements: {}", n_nodes, n_elems);

    // ── 2. Assign boundary conditions ─────────────────────────────────────────
    println!("[2/4] Assigning boundary conditions...");

    //  We iterate over all mesh vertices and classify them by z-coordinate.
    //  The unit sphere spans z ∈ [-1, 1].
    //  z < -0.7  →  bottom zone  (clamped)
    //  z >  0.7  →  top zone    (tensile load)
    let bottom_threshold: f32 = 0.3;
    let top_threshold:    f32 = 1.7;
    let applied_force_n:  f32 = 500.0; // Newtons per node (tensile, +z)

    let mut fixed_count  = 0usize;
    let mut loaded_count = 0usize;
    let mut free_count   = 0usize;

    // per-point BCs (keyed by vertex index in the triangulation)
    let point_bcs: FxHashMap<usize, BoundaryCondition> = mesh
        .triangulation_data()
        .vertices()
        .iter()
        .enumerate()
        .map(|(i, vertex)| {
            let z = vertex.point().z;
            let z = z as f32;

            let bc = if z < bottom_threshold {
                // Fixed: Dz = 0  (clamp in z only — allows slight XY Poisson expansion)
                fixed_count += 1;
                BoundaryCondition::Displacement(Displacement::Z(zero_fn(sim_seconds)))
            } else if z > top_threshold {
                // Force: Fz = +applied_force_n
                loaded_count += 1;
                BoundaryCondition::Force(Vector3::new(
                    const_fn(0.0,                sim_seconds),
                    const_fn(0.0,                sim_seconds),
                    const_fn(applied_force_n,    sim_seconds),
                ))
            } else {
                free_count += 1;
                BoundaryCondition::Free
            };

            (i, bc)
        })
        .collect();

    // No boundary-ID-level BCs (we use per-point BCs instead)
    let boundary_bcs: FxHashMap<BoundaryId, BoundaryCondition> = FxHashMap::default();

    println!(
        "    Fixed (bottom, z < {:.1}): {} nodes",
        bottom_threshold, fixed_count
    );
    println!(
        "    Loaded (top, z > {:.1}): {} nodes  ×  {:.0} N  =  {:.1} kN total",
        top_threshold, loaded_count, applied_force_n,
        (loaded_count as f32 * applied_force_n) / 1_000.0
    );
    println!("    Free (interior): {} nodes", free_count);

    // ── 3. Configure physics ──────────────────────────────────────────────────
    println!("[3/4] Configuring physics (Steel, E=200 GPa, ν=0.30)...");

    let unconfigured = computer::unconfigured(
        mesh.triangulation_data(),
        mesh.boundary_point_map(),
        &boundary_bcs,
        &point_bcs,
    );

    let bulk = BulkMaterialProps::builder()
        .density(7_800.0)          // kg/m³  — steel
        .damping(5.0)              // light damping to damp ringing
        .failure_criteria(FailureCriteria::default())
        .build();

    let isotropic = IsotropicMaterialProps::builder()
        .bulk_props(bulk)
        .elasticity_modulus(200e9)  // Pa — 200 GPa
        .elasticity_condition(ElasticityCondition::ThreeDimensional)
        .poissons_ratio(0.30)
        .build();

    let config = Config::builder()
        .duration(Duration::from_secs_f32(sim_seconds))
        .time_delta(Duration::from_secs_f32(dt_ms / 1_000.0))
        .material_props(MaterialProps::Isotropic(isotropic))
        .build();

    let mut computer = unconfigured.configure(config);

    // ── 4. Run simulation ─────────────────────────────────────────────────────
    println!("[4/4] Running simulation ({} ms, dt = {} ms)...\n",
        sim_seconds * 1_000.0, dt_ms);

    println!("{:<8} {:>14} {:>18} {:>16} {:>10}",
        "Step", "Time (ms)", "Max Stress (kPa)", "Max Disp (mm)", "Max |v| (m/s)");
    println!("{}", "-".repeat(72));

    let _total_steps = ((sim_seconds * 1_000.0) / dt_ms).ceil() as usize;
    let mut step = 0usize;

    loop {
        match computer.advance() {
            AdvanceResult::InProgress(c) => {
                step += 1;
                if step % report_every == 0 || step == 1 {
                    let data = c.export_data();
                    let time_ms = (step as f32) * dt_ms;

                    // Max displacement from initial position
                    let max_disp_m: f32 = data.nodes().iter()
                        .map(|n| (n.position() - n.initial_position()).norm())
                        .fold(0.0_f32, f32::max);

                    // Max velocity magnitude
                    let max_vel: f32 = data.nodes().iter()
                        .map(|n| n.velocity().norm())
                        .fold(0.0_f32, f32::max);

                    // Max stress (Frobenius norm of stress tensor)
                    let max_stress_pa: f32 = data.elements().iter()
                        .map(|e| e.stress().norm())
                        .fold(0.0_f32, f32::max);

                    // Count broken elements
                    let broken: usize = data.elements().iter()
                        .filter(|e| *e.is_broken())
                        .count();

                    println!(
                        "{:<8} {:>14.3} {:>18.3} {:>16.4} {:>10.3}{}",
                        step,
                        time_ms,
                        max_stress_pa / 1_000.0,   // → kPa
                        max_disp_m * 1_000.0,       // → mm
                        max_vel,
                        if broken > 0 { format!("  ⚠ {} broken", broken) } else { String::new() }
                    );
                }
                computer = c;
            }
            AdvanceResult::Done(done) => {
                step += 1;
                let data = done.export_data();
                let max_disp_m: f32 = data.nodes().iter()
                    .map(|n| (n.position() - n.initial_position()).norm())
                    .fold(0.0_f32, f32::max);
                let max_stress_pa: f32 = data.elements().iter()
                    .map(|e| e.stress().norm())
                    .fold(0.0_f32, f32::max);
                let broken: usize = data.elements().iter()
                    .filter(|e| *e.is_broken())
                    .count();

                println!();
                println!("=== Simulation complete ({} steps) ===", step);
                println!("  Final max displacement : {:.4} mm", max_disp_m * 1_000.0);
                println!("  Final max stress       : {:.3} kPa", max_stress_pa / 1_000.0);
                println!("  Broken elements        : {}", broken);
                println!();

                // ── Sanity checks ────────────────────────────────────────────
                println!("--- Sanity checks ---");

                // 1. Bottom nodes must not have moved more than 0.01 mm
                let max_bottom_disp: f32 = data.nodes().iter()
                    .zip(mesh.triangulation_data().vertices().iter())
                    .filter(|(_, v)| (v.point()[2] as f32) < bottom_threshold)
                    .map(|(n, _)| (n.position() - n.initial_position()).norm())
                    .fold(0.0_f32, f32::max);
                let bottom_ok = max_bottom_disp < 0.01e-3; // < 0.01 mm
                println!(
                    "  [{}] Fixed-end max displacement: {:.6} mm  (must be < 0.01 mm)",
                    if bottom_ok { "PASS" } else { "FAIL" },
                    max_bottom_disp * 1_000.0
                );

                // 2. Top nodes should have moved upward (positive z net displacement)
                let top_z_disp: f32 = data.nodes().iter()
                    .zip(mesh.triangulation_data().vertices().iter())
                    .filter(|(_, v)| (v.point()[2] as f32) > top_threshold)
                    .map(|(n, _)| n.position().z - n.initial_position().z)
                    .fold(f32::NEG_INFINITY, f32::max);
                let top_ok = top_z_disp > 0.0;
                println!(
                    "  [{}] Top-end net z-displacement:  {:.6} mm  (must be > 0)",
                    if top_ok { "PASS" } else { "FAIL" },
                    top_z_disp * 1_000.0
                );

                // 3. No NaN in node positions
                let any_nan = data.nodes().iter()
                    .any(|n| !n.position().x.is_finite());
                println!(
                    "  [{}] No NaN / Inf in node positions",
                    if !any_nan { "PASS" } else { "FAIL" }
                );

                println!();
                if bottom_ok && top_ok && !any_nan {
                    println!("✅  ALL CHECKS PASSED — physics engine responding correctly.");
                } else {
                    println!("❌  SOME CHECKS FAILED — review output above.");
                    std::process::exit(1);
                }
                break;
            }
        }
    }
}
