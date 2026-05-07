use super::always_open_window::AlwaysOpenWindow;
use egui::{Align, Context, Layout, RichText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Response {
    Noop,
    Close,
}

impl Response {
    pub fn closed(&self) -> bool {
        self == &Self::Close
    }
}

#[must_use]
pub fn show(err: &str, ctx: &Context) -> Response {
    let close = AlwaysOpenWindow::new("Error")
        .default_width(360.0)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(err).color(ui.style().visuals.error_fg_color));
            });
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                ui.button("Close").clicked()
            })
            .inner
        });
    if close {
        Response::Close
    } else {
        Response::Noop
    }
}
