#[macro_use]
pub mod app;
mod always_open_window;
pub mod boundary_conditions;
mod close_project_dialog;
mod delete_project_dialog;
mod dialog_utils;
pub mod drawing;
pub mod meshing;
pub mod simulation;
pub mod unicode_symbols;
pub mod premium;
pub mod gnomon;

mod project_handle;
use ecolor::Color32;
use egui::{Context, KeyboardShortcut, Ui};
pub use project_handle::ProjectHandle;

pub mod page;

pub mod error_dialog;

mod context_wrapper;
pub use context_wrapper::ContextWrapper;

mod plot_utils;

mod bottom_panel;

pub fn is_dark_mode(ctx: &Context) -> bool {
    ctx.style().visuals.dark_mode
}

pub fn on_primary_color(ctx: &Context) -> Color32 {
    if is_dark_mode(ctx) {
        Color32::WHITE
    } else {
        Color32::BLACK
    }
}

pub fn consume_shortcut(ui: &mut Ui, shortcut: &KeyboardShortcut) -> bool {
    ui.input_mut(|w| w.consume_shortcut(shortcut))
}
