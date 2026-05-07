use nalgebra::{Matrix3, Vector3};

fn main() {
    let c0 = Vector3::new(1.0, 2.0, 3.0);
    let c1 = Vector3::new(0.0, 1.0, 4.0);
    let c2 = Vector3::new(5.0, 6.0, 0.0);
    let m = Matrix3::from_columns(&[c0, c1, c2]);
    let inv = m.try_inverse().unwrap();

    let cross01 = c0.cross(&c1);
    let cross12 = c1.cross(&c2);
    let cross20 = c2.cross(&c0);
    let det = c2.dot(&cross01);
    let inv_det = 1.0 / det;

    let wgsl_inv = Matrix3::from_columns(&[
        Vector3::new(cross12.x, cross20.x, cross01.x) * inv_det,
        Vector3::new(cross12.y, cross20.y, cross01.y) * inv_det,
        Vector3::new(cross12.z, cross20.z, cross01.z) * inv_det,
    ]);

    println!("Nalgebra Inv:\n{}", inv);
    println!("WGSL Inv:\n{}", wgsl_inv);
}
