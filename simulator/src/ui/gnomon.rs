use egui_plot::{Line, PlotUi, PlotPoint};
use egui::Color32;
use std::f32::consts::PI;

pub fn draw_gnomon(ui: &mut PlotUi, rx: &mut f32, ry: &mut f32) {
    let project = |x: f32, y: f32, z: f32, rx: f32, ry: f32| -> [f64; 2] {
        let (sx, cx) = rx.sin_cos();
        let (sy, cy) = ry.sin_cos();

        // Rotate Y
        let nx = x * cy + z * sy;
        let nz = -x * sy + z * cy;

        // Rotate X
        let ny = y * cx - nz * sx;

        [nx as f64, ny as f64]
    };

    // Anchor gnomon at the bottom-left corner of the visible plot area
    let bounds = ui.plot_bounds();
    if !bounds.min()[0].is_finite() || !bounds.max()[0].is_finite() || 
       !bounds.min()[1].is_finite() || !bounds.max()[1].is_finite() {
        return;
    }
    let plot_w = bounds.max()[0] - bounds.min()[0];
    let plot_h = bounds.max()[1] - bounds.min()[1];
    
    // Size the axes to 12% of the smallest dimension
    let axis_len = plot_w.min(plot_h) * 0.12;
    
    // Place origin slightly inside the bottom-left corner
    let ox = bounds.min()[0] + plot_w * 0.05;
    let oy = bounds.min()[1] + plot_h * 0.05;

    let offset = |proj: [f64; 2]| -> [f64; 2] {
        [ox + proj[0] * axis_len, oy + proj[1] * axis_len]
    };

    let origin = offset(project(0.0, 0.0, 0.0, *rx, *ry));
    let x_tip  = offset(project(1.0, 0.0, 0.0, *rx, *ry));
    let y_tip  = offset(project(0.0, 1.0, 0.0, *rx, *ry));
    let z_tip  = offset(project(0.0, 0.0, 1.0, *rx, *ry));

    // Interaction logic
    if let Some(pointer_pos) = ui.pointer_coordinate() {
        let is_near = |p: [f64; 2], target: PlotPoint| {
            let dist_sq = (p[0] - target.x).powi(2) + (p[1] - target.y).powi(2);
            dist_sq < (axis_len * 0.15).powi(2)
        };

        if ui.response().clicked() {
            if is_near(x_tip, pointer_pos) {
                *rx = 0.0;
                *ry = PI / 2.0;
            } else if is_near(y_tip, pointer_pos) {
                *rx = PI / 2.0;
                *ry = 0.0;
            } else if is_near(z_tip, pointer_pos) {
                *rx = 0.0;
                *ry = 0.0;
            } else if is_near(origin, pointer_pos) {
                // Reset to isometric
                *rx = -0.5;
                *ry = 0.5;
            }
        }
    }

    // Also support dragging the gnomon area to rotate with primary button
    if ui.response().dragged_by(egui::PointerButton::Primary) {
        if let Some(press_origin) = ui.ctx().input(|i| i.pointer.press_origin()) {
            // Check if drag started in the gnomon's screen area
            let gnomon_screen_pos = ui.transform().position_from_point(&PlotPoint::new(ox, oy));
            let dist = (press_origin - gnomon_screen_pos).length();
            if dist < 60.0 { // 60 pixels radius around gnomon
                let delta = ui.response().drag_delta();
                *ry += delta.x * 0.01;
                *rx += delta.y * 0.01;
            }
        }
    }

    // Draw axes with numerical safety
    let draw_axis = |ui: &mut PlotUi, from: [f64; 2], to: [f64; 2], color: Color32, label: &str| {
        if (to[0] - from[0]).abs() > 1e-10 || (to[1] - from[1]).abs() > 1e-10 {
            if from[0].is_finite() && from[1].is_finite() && to[0].is_finite() && to[1].is_finite() {
                ui.line(Line::new(vec![from, to]).color(color).width(2.5));
                ui.text(egui_plot::Text::new(PlotPoint::new(to[0], to[1]),
                    egui::RichText::new(label).color(color).size(12.0).strong()));
            }
        }
    };
    
    draw_axis(ui, origin, x_tip, Color32::RED, "X");
    draw_axis(ui, origin, y_tip, Color32::GREEN, "Y");
    draw_axis(ui, origin, z_tip, Color32::BLUE, "Z");
}
