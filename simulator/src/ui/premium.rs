use egui::{Ui, RichText, Color32, Frame, Margin, Stroke, Rounding, epaint::Shadow, vec2, Rect, Sense, Separator};

/// Glassmorphism-style card with accent glow border and layered depth
pub fn premium_card<R>(ui: &mut Ui, title: &str, add_contents: impl FnOnce(&mut Ui) -> R) {
    let is_dark = ui.visuals().dark_mode;

    // Glassmorphism layered fill — semi-transparent so underlying content bleeds through
    let glass_fill = if is_dark {
        Color32::from_rgba_unmultiplied(24, 24, 32, 200)  // dark glass
    } else {
        Color32::from_rgba_unmultiplied(255, 255, 255, 180) // light glass
    };

    // Accent border — subtle indigo glow
    let accent = Color32::from_rgba_unmultiplied(99, 102, 241, 80);
    let border  = Color32::from_rgba_unmultiplied(99, 102, 241, 35);

    let frame_resp = Frame::none()
        .fill(glass_fill)
        .stroke(Stroke::new(1.0, border))
        .rounding(12.0)
        .inner_margin(Margin::symmetric(14.0, 12.0))
        .shadow(Shadow {
            offset: vec2(0.0, 4.0),
            blur: 16.0,
            spread: 0.0,
            color: Color32::from_rgba_premultiplied(0, 0, 0, if is_dark { 60 } else { 15 }),
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(|ui| {
                // --- Accent top bar ---
                let (bar_rect, _) = ui.allocate_exact_size(
                    vec2(ui.available_width(), 3.0),
                    Sense::hover(),
                );
                ui.painter().rect_filled(
                    Rect::from_min_size(bar_rect.min, vec2(40.0, 3.0)),
                    Rounding::same(3.0),
                    accent,
                );

                ui.add_space(6.0);
                ui.label(RichText::new(title).strong().size(13.5));
                ui.add_space(6.0);
                ui.add(Separator::default().spacing(0.0));
                ui.add_space(8.0);
                add_contents(ui);
            });
        });

    // Outer glow ring — painted on the painter *after* the frame for a halo effect
    let rect = frame_resp.response.rect.expand(1.0);
    ui.painter().rect_stroke(rect, 12.0, Stroke::new(1.0, Color32::from_rgba_unmultiplied(99, 102, 241, 18)));
}

/// Pulsing status dot — animates based on wall-clock time
pub fn status_dot(ui: &mut Ui, active: bool) {
    let t = ui.ctx().input(|i| i.time) as f32;
    let pulse = ((t * 2.5).sin() * 0.5 + 0.5) as f32; // 0..1 oscillation at ~2.5Hz
    let (base_r, base_g, base_b) = if active { (74, 222, 128) } else { (156, 163, 175) }; // green / gray
    let alpha = if active { (140.0 + pulse * 115.0) as u8 } else { 180u8 };
    let color = Color32::from_rgba_unmultiplied(base_r, base_g, base_b, alpha);
    let (rect, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
    ui.painter().circle_filled(rect.center(), 5.0, color);
    if active {
        // Outer ripple ring
        let ring_r = 5.0 + pulse * 4.0;
        let ring_alpha = ((1.0 - pulse) * 80.0) as u8;
        ui.painter().circle_stroke(
            rect.center(), ring_r,
            Stroke::new(1.5, Color32::from_rgba_unmultiplied(74, 222, 128, ring_alpha)),
        );
        ui.ctx().request_repaint();
    }
}
