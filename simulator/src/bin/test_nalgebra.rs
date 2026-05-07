use nalgebra::Matrix3;

fn main() {
    let m = Matrix3::new(
        1.0, 2.0, 3.0,
        4.0, 5.0, 6.0,
        7.0, 8.0, 9.0
    );
    println!("Matrix:\n{}", m);
    println!("Column sum:\n{}", m.column_sum());
    println!("Row sum:\n{}", m.row_sum());
}
