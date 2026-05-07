// Compute shader for DPM force calculation — Pass 2
// Reads stress written by update_elements.wgsl (Pass 1).
// Each invocation computes the 4 nodal forces for one element and
// writes them into an OUTPUT buffer (4 vec4<f32> per element).
// The CPU scatters these into per-node totals — no atomic overflow.

struct GpuNode {
    initial_position: vec4<f32>,
    position:         vec4<f32>,
    velocity:         vec4<f32>,
    mass:             f32,
    _pad0: f32, _pad1: f32, _pad2: f32,
};

// Must match GpuElement in lib.rs and update_elements.wgsl exactly.
struct GpuElement {
    node_indices:       vec4<u32>,
    stress_col0:        vec4<f32>,
    stress_col1:        vec4<f32>,
    stress_col2:        vec4<f32>,
    is_broken:          u32,
    strain_energy_bits: u32,
    _pad0: u32,
    _pad1: u32,
};

// Output: 4 force vectors per element (for nodes a, b, c, d).
// Stored as 4 consecutive vec4<f32> per element (w component unused).
struct ElementForces {
    force_a: vec4<f32>,
    force_b: vec4<f32>,
    force_c: vec4<f32>,
    force_d: vec4<f32>,
};

@group(0) @binding(0) var<storage, read>       nodes:         array<GpuNode>;
@group(0) @binding(1) var<storage, read>       elements:      array<GpuElement>;
@group(0) @binding(2) var<storage, read_write> element_forces: array<ElementForces>;

fn inverse3x3(m: mat3x3<f32>) -> mat3x3<f32> {
    let c0 = m[0]; let c1 = m[1]; let c2 = m[2];
    let row0 = cross(c1, c2);
    let row1 = cross(c2, c0);
    let row2 = cross(c0, c1);
    let det    = dot(c0, row0);
    let invDet = 1.0 / det;
    return mat3x3<f32>(
        vec3<f32>(row0.x, row1.x, row2.x) * invDet,
        vec3<f32>(row0.y, row1.y, row2.y) * invDet,
        vec3<f32>(row0.z, row1.z, row2.z) * invDet,
    );
}

fn determinant3x3(m: mat3x3<f32>) -> f32 {
    return dot(m[0], cross(m[1], m[2]));
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if idx >= arrayLength(&elements) { return; }

    let element = elements[idx];

    // Broken elements contribute zero force
    if element.is_broken == 1u {
        element_forces[idx].force_a = vec4<f32>(0.0);
        element_forces[idx].force_b = vec4<f32>(0.0);
        element_forces[idx].force_c = vec4<f32>(0.0);
        element_forces[idx].force_d = vec4<f32>(0.0);
        return;
    }

    let n0 = nodes[element.node_indices.x];
    let n1 = nodes[element.node_indices.y];
    let n2 = nodes[element.node_indices.z];
    let n3 = nodes[element.node_indices.w];

    // Reference configuration
    let r_ba = n1.initial_position.xyz - n0.initial_position.xyz;
    let r_ca = n2.initial_position.xyz - n0.initial_position.xyz;
    let r_da = n3.initial_position.xyz - n0.initial_position.xyz;
    let r    = mat3x3<f32>(r_ba, r_ca, r_da);

    let volume = abs(determinant3x3(r)) / 6.0;
    let h      = inverse3x3(r);

    // Current configuration
    let d_ba = n1.position.xyz - n0.position.xyz;
    let d_ca = n2.position.xyz - n0.position.xyz;
    let d_da = n3.position.xyz - n0.position.xyz;
    let d    = mat3x3<f32>(d_ba, d_ca, d_da);

    let f_mat = d * h;

    // Stress written by Pass 1 (column-major)
    let stress = mat3x3<f32>(
        element.stress_col0.xyz,
        element.stress_col1.xyz,
        element.stress_col2.xyz,
    );

    let p = f_mat * stress;

    // Shape function gradients = rows of H = columns of Hᵀ
    let h_t    = transpose(h);
    let grad_b = h_t[0];
    let grad_c = h_t[1];
    let grad_d = h_t[2];

    let force_b = -volume * (p * grad_b);
    let force_c = -volume * (p * grad_c);
    let force_d = -volume * (p * grad_d);
    let force_a = -(force_b + force_c + force_d);

    // Write per-element forces — CPU scatters into node totals
    element_forces[idx].force_a = vec4<f32>(force_a, 0.0);
    element_forces[idx].force_b = vec4<f32>(force_b, 0.0);
    element_forces[idx].force_c = vec4<f32>(force_c, 0.0);
    element_forces[idx].force_d = vec4<f32>(force_d, 0.0);
}
