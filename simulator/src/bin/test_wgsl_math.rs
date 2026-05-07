use nalgebra::{Matrix3, Vector3};

// WGSL translated logic
fn cross(a: Vector3<f32>, b: Vector3<f32>) -> Vector3<f32> {
    a.cross(&b)
}

fn dot(a: Vector3<f32>, b: Vector3<f32>) -> f32 {
    a.dot(&b)
}

fn wgsl_inverse(m: Matrix3<f32>) -> Matrix3<f32> {
    let c0: Vector3<f32> = m.column(0).into();
    let c1: Vector3<f32> = m.column(1).into();
    let c2: Vector3<f32> = m.column(2).into();
    let row0 = c1.cross(&c2);
    let row1 = c2.cross(&c0);
    let row2 = c0.cross(&c1);
    let det = c0.dot(&row0);
    let inv_det = 1.0 / det;

    // Build column by column: column j = (row0[j], row1[j], row2[j])
    Matrix3::from_columns(&[
        Vector3::new(row0.x, row1.x, row2.x) * inv_det,
        Vector3::new(row0.y, row1.y, row2.y) * inv_det,
        Vector3::new(row0.z, row1.z, row2.z) * inv_det,
    ])
}

fn main() {
    // Test exact element 3 data
    let r = Matrix3::new(
        -0.2286224,  -0.7163689, -0.25749978,
        -0.4294319,  -0.5002476,  0.3565007,
         0.32288677,  0.4088394, -0.23615825
    );
    let d = r;
    let stress = Matrix3::new(
         35.99613,  0.5726507,   8.83506,
       0.57539606,  41.523537, 3.0679967,
         8.847325,  3.0784779,  70.76388
    );
    let h_nalgebra = r.try_inverse().unwrap();
    let volume_nalgebra = (r.determinant() as f32).abs() / 6.0;
    let f_nalgebra = d * h_nalgebra;
    let p_nalgebra = f_nalgebra * stress;
    let force_b_nalgebra = -volume_nalgebra * (p_nalgebra * h_nalgebra.transpose().column(0).into_owned());

    // WGSL
    let h_wgsl = wgsl_inverse(r);
    let volume_wgsl = r.determinant().abs() / 6.0; // Assume det matches
    let f_wgsl = d * h_wgsl;
    let p_wgsl = f_wgsl * stress;
    let grad_b_wgsl = h_wgsl.transpose().column(0).into_owned();
    let force_b_wgsl = -volume_wgsl * (p_wgsl * grad_b_wgsl);

    println!("Nalgebra Force B:\n{}", force_b_nalgebra);
    println!("WGSL Force B:\n{}", force_b_wgsl);
    println!("Diff:\n{}", force_b_nalgebra - force_b_wgsl);
}
