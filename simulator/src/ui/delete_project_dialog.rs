use super::{always_open_window::AlwaysOpenWindow, unicode_symbols, ProjectHandle};
use egui::{Align, Context, Layout, RichText};
use serde::{Deserialize, Serialize};
use std::ffi::OsString;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    project_name: OsString,
    project_handle: ProjectHandle,
}

impl State {
    pub fn new(project_name: OsString, project_handle: ProjectHandle) -> Self {
        Self {
            project_name,
            project_handle,
        }
    }
}

pub enum Response {
    Noop,
    Delete(ProjectHandle),
    Cancel,
}

pub fn show(state: &mut State, ctx: &Context) -> Response {
    AlwaysOpenWindow::new("Delete project").show(ctx, |ui| {
        ui.add_space(8.0);
        ui.label(
            RichText::new(format!(
                "{} Are you sure you want to delete this project {}",
                unicode_symbols::WARNING,
                state.project_name.to_string_lossy()
            ))
            .heading(),
        );
        ui.add_space(8.0);
        ui.with_layout(
            Layout::right_to_left(Align::Min).with_main_justify(false),
            |ui| {
                if ui.button("Cancel").clicked() {
                    Response::Cancel
                } else if ui.button("Delete").clicked() {
                    Response::Delete(state.project_handle)
                } else {
                    Response::Noop
                }
            },
        )
        .inner
    })
}
