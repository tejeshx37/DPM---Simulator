// GPU Pass 1 — Element State Update
// Computes Green-Lagrange strain, stress (isotropic or orthotropic),
// strain energy, and fracture detection. Writes results into the
// mutable GpuElement buffer so Pass 2 (force accumulation) can read
// the freshly-computed stress without any CPU round-trip.

// ── Node buffer (read-only) ──────────────────────────────────────────────────
struct GpuNode {
    initial_position: vec4<f32>,
    position:         vec4<f32>,
    velocity:         vec4<f32>,
    mass:             f32,
    _pad0: f32, _pad1: f32, _pad2: f32,
};

// ── Element buffer (read-write) ──────────────────────────────────────────────
// Must match GpuElement in lib.rs exactly (byte-for-byte).
struct GpuElement {
    node_indices:  vec4<u32>,
    stress_col0:   vec4<f32>,  // column 0 of stress matrix (+ pad)
    stress_col1:   vec4<f32>,  // column 1
    stress_col2:   vec4<f32>,  // column 2
    // u32 for is_broken (0 or 1); f32 for strain_energy; 2× u32 padding
    is_broken:     u32,
    strain_energy_bits: u32,   // bitcast of f32 strain_energy
    is_inverted:   u32,
    _pad1: u32,
};

// ── Material uniform ─────────────────────────────────────────────────────────
struct GpuMaterial {
    // Shared / isotropic
    density:              f32,
    damping:              f32,
    failure_strain_energy: f32,    // 0 → disabled
    failure_tensile:      f32,     // 0 → disabled
    failure_compressive:  f32,     // 0 → disabled
    // 0 = isotropic, 1 = orthotropic
    material_type:        u32,
    // Isotropic params
    elasticity_modulus:   f32,
    poissons_ratio:       f32,
    // Orthotropic params (9 Poisson ratios + 3 moduli + 3 shear)
    ex: f32, ey: f32, ez: f32,
    nu_xy: f32, nu_yx: f32,
    nu_yz: f32, nu_zy: f32,
    nu_zx: f32, nu_xz: f32,
    g_xy: f32, g_yz: f32, g_zx: f32,
    _pad0: f32, _pad1: f32, _pad2: f32,
};

@group(0) @binding(0) var<storage, read>       nodes:    array<GpuNode>;
@group(0) @binding(1) var<storage, read_write>  elements: array<GpuElement>;
@group(0) @binding(2) var<uniform>              material: GpuMaterial;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn inverse3x3(m: mat3x3<f32>) -> mat3x3<f32> {
    let c0 = m[0]; let c1 = m[1]; let c2 = m[2];
    let row0 = cross(c1, c2);
    let row1 = cross(c2, c0);
    let row2 = cross(c0, c1);
    let det    = dot(c0, row0);
    let inv    = 1.0 / det;
    return mat3x3<f32>(
        vec3<f32>(row0.x, row1.x, row2.x) * inv,
        vec3<f32>(row0.y, row1.y, row2.y) * inv,
        vec3<f32>(row0.z, row1.z, row2.z) * inv,
    );
}

fn determinant3x3(m: mat3x3<f32>) -> f32 {
    return dot(m[0], cross(m[1], m[2]));
}

// Voigt notation: [e11, e22, e33, 2e23, 2e13, 2e12]
fn strain_to_voigt(e: mat3x3<f32>) -> array<f32, 6> {
    var v: array<f32, 6>;
    v[0] = e[0][0];
    v[1] = e[1][1];
    v[2] = e[2][2];
    v[3] = e[1][2] * 2.0;
    v[4] = e[0][2] * 2.0;
    v[5] = e[0][1] * 2.0;
    return v;
}

// Rebuild symmetric 3×3 stress matrix from Voigt stress vector
// [s11, s22, s33, s23, s13, s12]
fn voigt_to_stress(s: array<f32, 6>) -> mat3x3<f32> {
    // Column-major in WGSL: mat3x3(col0, col1, col2)
    return mat3x3<f32>(
        vec3<f32>(s[0], s[5], s[4]),   // col 0: [s11, s12, s13]
        vec3<f32>(s[5], s[1], s[3]),   // col 1: [s12, s22, s23]
        vec3<f32>(s[4], s[3], s[2]),   // col 2: [s13, s23, s33]
    );
}

// ── Isotropic stress via C·ε ─────────────────────────────────────────────────
fn isotropic_stress(e_mod: f32, nu_raw: f32, strain: mat3x3<f32>) -> mat3x3<f32> {
    let nu = clamp(nu_raw, 0.0, 0.499);
    let factor = e_mod / ((1.0 + nu) * (1.0 - 2.0 * nu));
    let diag   = 1.0 - nu;
    let off    = nu;
    let shear  = (1.0 - 2.0 * nu) / 2.0;

    var sv = strain_to_voigt(strain);
    var s: array<f32, 6>;
    s[0] = factor * (diag * sv[0] + off  * sv[1] + off  * sv[2]);
    s[1] = factor * (off  * sv[0] + diag * sv[1] + off  * sv[2]);
    s[2] = factor * (off  * sv[0] + off  * sv[1] + diag * sv[2]);
    s[3] = factor * shear * sv[3];
    s[4] = factor * shear * sv[4];
    s[5] = factor * shear * sv[5];
    return voigt_to_stress(s);
}

// ── Orthotropic stress — analytical stiffness ────────────────────────────────
// The 6×6 orthotropic compliance matrix S is block-diagonal:
//
//   S_normal (3×3):  [ 1/Ex   -νyx/Ey  -νzx/Ez ]
//                    [ -νxy/Ex  1/Ey   -νzy/Ez ]
//                    [ -νxz/Ex -νyz/Ey   1/Ez  ]
//
//   S_shear (3×3 diag): [ 1/Gyz, 1/Gzx, 1/Gxy ]
//
// We invert S_normal analytically (3×3 cofactor formula) and take
// the reciprocal of the three shear terms.  All indexing is by
// literal constants — no runtime-variable matrix indexing.
fn orthotropic_stress(strain: mat3x3<f32>) -> mat3x3<f32> {
    let ex  = material.ex;  let ey  = material.ey;  let ez  = material.ez;
    let nxy = material.nu_xy; let nyx = material.nu_yx;
    let nyz = material.nu_yz; let nzy = material.nu_zy;
    let nzx = material.nu_zx; let nxz = material.nu_xz;
    let gxy = material.g_xy; let gyz = material.g_yz; let gzx = material.g_zx;

    // ── Build S_normal (row-major for readability) ────────────────────────
    // S[row][col], stored as 9 scalars
    let s00 =  1.0/ex;    let s01 = -nyx/ey;   let s02 = -nzx/ez;
    let s10 = -nxy/ex;   let s11 =  1.0/ey;    let s12 = -nzy/ez;
    let s20 = -nxz/ex;   let s21 = -nyz/ey;   let s22 =  1.0/ez;

    // ── Determinant of S_normal (Sarrus) ─────────────────────────────────
    let det = s00*(s11*s22 - s12*s21)
            - s01*(s10*s22 - s12*s20)
            + s02*(s10*s21 - s11*s20);
    let inv_det = 1.0 / det;

    // ── Cofactor matrix (= adjugate transposed, but S is symmetric) ───────
    let d00 = ( s11*s22 - s12*s21) * inv_det;
    let d01 = (-s01*s22 + s02*s21) * inv_det;
    let d02 = ( s01*s12 - s02*s11) * inv_det;
    let d10 = (-s10*s22 + s12*s20) * inv_det;
    let d11 = ( s00*s22 - s02*s20) * inv_det;
    let d12 = (-s00*s12 + s02*s10) * inv_det;
    let d20 = ( s10*s21 - s11*s20) * inv_det;
    let d21 = (-s00*s21 + s01*s20) * inv_det;
    let d22 = ( s00*s11 - s01*s10) * inv_det;

    // ── Voigt strain vector ───────────────────────────────────────────────
    // ev = [e11, e22, e33, 2e23, 2e13, 2e12]
    var ev = strain_to_voigt(strain);

    // ── Stress Voigt vector via D · ev ────────────────────────────────────
    // Normal block (rows 0-2, cols 0-2):
    let s0 = d00*ev[0] + d01*ev[1] + d02*ev[2];
    let s1 = d10*ev[0] + d11*ev[1] + d12*ev[2];
    let s2 = d20*ev[0] + d21*ev[1] + d22*ev[2];
    // Shear block (diagonal, rows 3-5):
    let s3 = gyz * ev[3];
    let s4 = gzx * ev[4];
    let s5 = gxy * ev[5];

    var s: array<f32, 6>;
    s[0] = s0; s[1] = s1; s[2] = s2;
    s[3] = s3; s[4] = s4; s[5] = s5;
    return voigt_to_stress(s);
}

// ── Simple principal stress bounds (Gershgorin, constant indexing) ───────────
fn principal_stress_bounds(stress: mat3x3<f32>) -> vec2<f32> {
    // mat3x3 is column-major: stress[col][row].
    // Diagonal elements: stress[0][0], stress[1][1], stress[2][2]
    let d0 = stress[0][0];
    let d1 = stress[1][1];
    let d2 = stress[2][2];
    let max_pos = max(max(d0, d1), d2);
    let min_neg = min(min(d0, d1), d2);
    return vec2<f32>(max_pos, min_neg);
}


// ── Strain energy density ────────────────────────────────────────────────────
fn strain_energy_density(stress: mat3x3<f32>, strain: mat3x3<f32>) -> f32 {
    // Frobenius inner product: sum_ij(sigma_ij * epsilon_ij) * 0.5
    // mat3x3 is column-major in WGSL: m[col][row], so dot of columns is correct.
    let s0 = dot(stress[0], strain[0]);
    let s1 = dot(stress[1], strain[1]);
    let s2 = dot(stress[2], strain[2]);
    return (s0 + s1 + s2) * 0.5;
}

// ── Main ──────────────────────────────────────────────────────────────────────
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if idx >= arrayLength(&elements) { return; }

    // Already broken — keep broken, skip computation
    if elements[idx].is_broken == 1u { return; }

    let el = elements[idx];
    let n0 = nodes[el.node_indices.x];
    let n1 = nodes[el.node_indices.y];
    let n2 = nodes[el.node_indices.z];
    let n3 = nodes[el.node_indices.w];

    // Reference edge vectors
    let r_ba = n1.initial_position.xyz - n0.initial_position.xyz;
    let r_ca = n2.initial_position.xyz - n0.initial_position.xyz;
    let r_da = n3.initial_position.xyz - n0.initial_position.xyz;
    let R    = mat3x3<f32>(r_ba, r_ca, r_da);   // column-major

    let det_R = determinant3x3(R);
    if abs(det_R) < 1e-9 {
        // Degenerate — mark broken
        elements[idx].is_broken = 1u;
        return;
    }

    // Current edge vectors
    let d_ba = n1.position.xyz - n0.position.xyz;
    let d_ca = n2.position.xyz - n0.position.xyz;
    let d_da = n3.position.xyz - n0.position.xyz;
    let D    = mat3x3<f32>(d_ba, d_ca, d_da);

    // Deformation gradient F = D · R⁻¹
    let H = inverse3x3(R);
    var F = D * H;

    if determinant3x3(F) <= 0.0 {
        elements[idx].is_inverted = 1u;
        F = mat3x3<f32>(
            vec3<f32>(1.0, 0.0, 0.0),
            vec3<f32>(0.0, 1.0, 0.0),
            vec3<f32>(0.0, 0.0, 1.0),
        );
    } else {
        elements[idx].is_inverted = 0u;
    }

    // Green-Lagrange strain E = ½(Fᵀ·F − I)
    let Ft = transpose(F);
    let C  = Ft * F;
    let E  = (C - mat3x3<f32>(
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(0.0, 0.0, 1.0),
    )) * 0.5;

    // Stress from material model
    var stress: mat3x3<f32>;
    if material.material_type == 0u {
        stress = isotropic_stress(material.elasticity_modulus, material.poissons_ratio, E);
    } else {
        stress = orthotropic_stress(E);
    }

    // Strain energy density
    let se = strain_energy_density(stress, E);

    // Failure check
    var broken = false;
    if material.failure_strain_energy > 0.0 && se >= material.failure_strain_energy {
        broken = true;
    }
    if !broken && (material.failure_tensile > 0.0 || material.failure_compressive > 0.0) {
        let bounds = principal_stress_bounds(stress);
        if material.failure_tensile > 0.0 && bounds.x >= material.failure_tensile {
            broken = true;
        }
        if !broken && material.failure_compressive > 0.0 && abs(bounds.y) >= material.failure_compressive {
            broken = true;
        }
    }

    // Write back results
    elements[idx].stress_col0 = vec4<f32>(stress[0].x, stress[0].y, stress[0].z, 0.0);
    elements[idx].stress_col1 = vec4<f32>(stress[1].x, stress[1].y, stress[1].z, 0.0);
    elements[idx].stress_col2 = vec4<f32>(stress[2].x, stress[2].y, stress[2].z, 0.0);
    elements[idx].strain_energy_bits = bitcast<u32>(se);
    elements[idx].is_broken = select(0u, 1u, broken);
}
