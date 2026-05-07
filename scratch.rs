use std::f64::consts::PI;

fn main() {
    let num_points = 512;
    let radius = 1.0;
    let n = num_points as f64;
    let phi = PI * (5.0f64.sqrt() - 1.0);

    let num_shells = (n.powf(1.0 / 3.0)) as u32;
    let num_shells = num_shells.max(1);
    let points_per_shell = num_points / num_shells;

    println!("num_shells: {}, points_per_shell: {}", num_shells, points_per_shell);

    let mut points = vec![];

    for shell in 1..=num_shells {
        let r_shell = radius * (shell as f64 / num_shells as f64);
        let n_shell = if shell == num_shells {
            num_points - (num_shells - 1) * points_per_shell
        } else {
            points_per_shell
        };

        for i in 0..n_shell {
            let y = 1.0 - (i as f64 / (n_shell as f64 - 1.0).max(1.0)) * 2.0;
            let r = (1.0 - y * y).max(0.0).sqrt();

            let theta = phi * i as f64;
            let x = theta.cos() * r;
            let z = theta.sin() * r;

            points.push((x * r_shell, y * r_shell, z * r_shell));
            if x.is_nan() || y.is_nan() || z.is_nan() {
                println!("NaN at shell {}, i {}", shell, i);
            }
        }
    }
    
    println!("Total points generated: {}", points.len());
    // Check if points are identical or extremely close
    let mut min_dist = f64::MAX;
    for i in 0..points.len() {
        for j in i+1..points.len() {
            let dx = points[i].0 - points[j].0;
            let dy = points[i].1 - points[j].1;
            let dz = points[i].2 - points[j].2;
            let dist = (dx*dx + dy*dy + dz*dz).sqrt();
            if dist < min_dist {
                min_dist = dist;
            }
        }
    }
    println!("Minimum distance between points: {}", min_dist);
}
