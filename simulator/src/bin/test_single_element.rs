use nalgebra::{Matrix3, Vector3};

fn main() {
    // WGSL logic
    let n0_p = Vector3::new(0.0, 0.0, 0.0);
    let n1_p = Vector3::new(1.0, 0.0, 0.0);
    let n2_p = Vector3::new(0.0, 1.0, 0.0);
    let n3_p = Vector3::new(0.0, 0.0, 1.0);

    let n0_ip = Vector3::new(0.0, 0.0, 0.0);
    let n1_ip = Vector3::new(1.0, 0.0, 0.0);
    let n2_ip = Vector3::new(0.0, 1.0, 0.0);
    let n3_ip = Vector3::new(0.0, 0.0, 1.0);

    let r_ba = n1_ip - n0_ip;
    let r_ca = n2_ip - n0_ip;
    let r_da = n3_ip - n0_ip;
    let r = Matrix3::from_columns(&[r_ba, r_ca, r_da]);

    let d_ba = n1_p - n0_p;
    let d_ca = n2_p - n0_p;
    let d_da = n3_p - n0_p;
    let d = Matrix3::from_columns(&[d_ba, d_ca, d_da]);

    let h = r.try_inverse().unwrap();
    let volume = (r.determinant() as f32).abs() / 6.0;

    let f_mat = d * h;

    let stress = Matrix3::new(
        1.0, 2.0, 3.0,
        2.0, 4.0, 5.0,
        3.0, 5.0, 6.0,
    );

    let p = f_mat * stress;

    let h_t = h.transpose();
    let grad_b = h_t.column(0).into_owned();
    let grad_c = h_t.column(1).into_owned();
    let grad_d = h_t.column(2).into_owned();

    let force_b = -volume * (p * grad_b);
    let force_c = -volume * (p * grad_c);
    let force_d = -volume * (p * grad_d);
    let force_a = -(force_b + force_c + force_d);

    println!("CPU force_a: {:?}", force_a);
    println!("CPU force_b: {:?}", force_b);
    println!("CPU force_c: {:?}", force_c);
    println!("CPU force_d: {:?}", force_d);
}
