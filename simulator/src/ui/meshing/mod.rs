mod dialog;

use super::{
    bottom_panel, error_dialog, gnomon, plot_utils, unicode_symbols,
    ContextWrapper,
};
use crate::model::{
    mesh_generator::{MeshGenerator, State},
    project::data::{Data, WithBoundaryConditions, WithMesh, WithShape},
    state_channel::{self, Receiver, STReceiver, Sender},
};
use cgal::triangulation;
use egui::{CentralPanel, Color32, Frame, Rounding, Ui};
use egui_plot::{Line, PlotUi, Points, Polygon};
use mesh::{Constraint, Mesh};
use rayon::prelude::*;

#[derive(Debug)]
pub struct Page {
    mesh_generator: MeshGenerator<ContextWrapper>,
    dialog_state: Option<dialog::State>,
    input_error: Option<String>,
    show_wireframe_only: bool,
    hide_mesh: bool,
    show_constraints: bool,
    show_interior_points: bool,
    state_receiver: STReceiver<State>,
    error_receiver: Receiver<String, Option<String>>,
    rotation_x: f32,
    rotation_y: f32,
    slice_z: f32,
    slice_enabled: bool,
    needs_reset: bool,
    auto_rotate: bool,
    run_simulation_clicked: bool,
}

#[derive(Debug)]
pub enum MenuResponse {
    Noop(Page),
    EditBoundaryConditions(Data<WithBoundaryConditions>),
    EditShape(Data<WithShape>),
}

#[derive(Debug)]
pub enum Response {
    Noop(Page),
    RunSimulation(Data<WithMesh>),
}

impl From<Data<WithBoundaryConditions>> for Page {
    fn from(project_data: Data<WithBoundaryConditions>) -> Self {
        Self::with_mesh_generator(|state_sender, error_sender| {
            MeshGenerator::new(project_data, state_sender, error_sender)
        })
    }
}

impl From<Data<WithMesh>> for Page {
    fn from(project_data: Data<WithMesh>) -> Self {
        Self::with_mesh_generator(|state_sender, error_sender| {
            MeshGenerator::new_with_mesh(project_data, state_sender, error_sender)
                .expect("State channel is active")
        })
    }
}

impl Page {
    fn with_mesh_generator(
        mesh_generator: impl FnOnce(Sender<State>, Sender<String>) -> MeshGenerator<ContextWrapper>,
    ) -> Self {
        let (state_sender, state_receiver) = state_channel::same_type_with_default(32);
        let (error_sender, error_receiver) = state_channel::with_default(1);
        Self {
            mesh_generator: mesh_generator(state_sender, error_sender),
            dialog_state: None,
            input_error: None,
            show_wireframe_only: true,
            hide_mesh: false,
            show_constraints: true,
            show_interior_points: true,
            state_receiver,
            error_receiver,
            rotation_x: -0.5,
            rotation_y: 0.5,
            slice_z: 0.0,
            slice_enabled: false,
            needs_reset: true,
            auto_rotate: false,
            run_simulation_clicked: false,
        }
    }

    #[must_use]
    pub fn add_menu_items(self, ui: &mut Ui) -> MenuResponse {
        puffin::profile_function!();
        #[derive(Debug, Default)]
        struct Response {
            edit_bc: bool,
            edit_shape: bool,
        }
        let opt = ui
            .menu_button("Edit", |ui| {
                let mut response = Response::default();
                if ui.button("Edit conditions").clicked() {
                    response.edit_bc = true;
                    ui.close_menu();
                }
                if ui.button("Edit shape").clicked() {
                    response.edit_shape = true;
                    ui.close_menu();
                }
                response
            })
            .inner;
        let Some(response) = opt else {
            return MenuResponse::Noop(self);
        };
        if response.edit_bc {
            MenuResponse::EditBoundaryConditions(self.mesh_generator.project_data_with_bc())
        } else if response.edit_shape {
            MenuResponse::EditShape(
                self.mesh_generator
                    .project_data_with_bc()
                    .without_boundary_conditions()
                    .0,
            )
        } else {
            MenuResponse::Noop(self)
        }
    }

    #[must_use]
    pub fn add_contents(mut self, ui: &mut Ui) -> Response {
        puffin::profile_function!();
        self.mesh_generator.set_refresh_token(ui.ctx());
        if self.state_receiver.update().is_err() {
            // Worker thread died, show error if not already handled
            if self.error_receiver.data.is_none() {
                 self.error_receiver.data = Some(String::from("Meshing worker thread disconnected unexpectedly."));
            }
        }
        let _ = self.error_receiver.update();

        bottom_panel::show("meshing_bottom_panel", ui, |ui| self.add_bottom_panel(ui));

        self.add_physics_preview_sidebar(ui);

        CentralPanel::default()
            .frame(Frame::default())
            .show_inside(ui, |ui| {
                let is_mesh = matches!(self.state_receiver.data, State::Mesh(_));
                let auto = self.needs_reset;
                
                if self.needs_reset {
                    self.needs_reset = false;
                }
                
                let plot_response = plot_utils::plot_without_clutter("meshing_preview_plot")
                    .auto_bounds(egui::Vec2b::new(auto, auto))
                    .show(ui, |ui| {
                        if ui.response().dragged_by(egui::PointerButton::Secondary) {
                            let delta = ui.response().drag_delta();
                            self.rotation_y += delta.x * 0.01;
                            self.rotation_x += delta.y * 0.01;
                        }
                        if self.auto_rotate {
                            self.rotation_y += 0.01;
                            ui.ctx().request_repaint();
                        }
                        self.plot_contents_optimized(ui);
                        if !auto {
                            gnomon::draw_gnomon(ui, &mut self.rotation_x, &mut self.rotation_y);
                        }
                    });
                if plot_response.response.clicked() && is_mesh {
                    // self.selected_point = ...
                }
            });

        if let Some(err) = self.error_receiver.data.as_ref() {
            if error_dialog::show(err, ui.ctx()).closed() {
                self.error_receiver.data = None;
            }
        }

        let solid_bounds = (!self.mesh_generator.polygon_data().polyhedron_set().is_empty())
            .then(|| self.mesh_generator.polygon_data().polyhedron_vertex_axis_bounds())
            .flatten();
        let data = Self::input_dialog_and_error_ui(
            ui,
            &mut self.dialog_state,
            &mut self.input_error,
            solid_bounds,
        );
        if let Some(data) = data {
            self.mesh_generator
                .generate(
                    data.num_points,
                    data.size_bound_override,
                    data.thickness,
                    data.seeding_config,
                )
                .expect("Worker thread is active");
        }

        let should_run = matches!(self.state_receiver.data, State::Mesh(_))
            && (self.run_simulation_clicked || ui.input(|i| i.key_pressed(egui::Key::Enter)));

        let response = if should_run {
            let mesh = if let State::Mesh(mesh) = &self.state_receiver.data {
                mesh.clone()
            } else {
                unreachable!()
            };
            Response::RunSimulation(self.mesh_generator.project_data_with_bc().with_mesh(mesh))
        } else {
            Response::Noop(self)
        };

        response
    }

    fn plot_contents_optimized(&self, ui: &mut PlotUi) {
        puffin::profile_function!();
        let state = self.state_receiver.data.clone();
        match state {
            State::Idle | State::GeneratingMesh(_) => {
                plot_utils::plot_solid_geometry(ui, self.mesh_generator.polygon_data(), self.rotation_x, self.rotation_y);
            }
            State::Mesh(mesh) => {
                let projector = plot_utils::Projector::new(self.rotation_x, self.rotation_y);
                self.plot_mesh_optimized(ui, &mesh, &projector);
            }
        }
    }

    fn plot_mesh_optimized(&self, ui: &mut PlotUi, mesh: &Mesh, projector: &plot_utils::Projector) {
        puffin::profile_function!();
        let data = mesh.triangulation_data();
        
        // Parallel projection of all mesh vertices
        let projected_vertices: Vec<[f64; 2]> = data.vertices()
            .par_iter()
            .map(|v| {
                let p = projector.project([v.point().x, v.point().y, v.point().z]);
                [p[0], p[1]]
            })
            .collect();

        if !self.hide_mesh {
            let stroke_color = super::on_primary_color(ui.ctx());
            if self.show_wireframe_only {
                data.edges().iter().for_each(|pair| {
                    if self.slice_enabled {
                        let p1_z = data.vertices()[pair.0].point().z;
                        let p2_z = data.vertices()[pair.1].point().z;
                        if p1_z < self.slice_z && p2_z < self.slice_z { return; }
                    }
                    let p1 = projected_vertices[pair.0];
                    let p2 = projected_vertices[pair.1];
                    
                    // Guard against non-finite or degenerate lines
                    if p1[0].is_finite() && p1[1].is_finite() && p2[0].is_finite() && p2[1].is_finite() {
                        if (p2[0] - p1[0]).abs() > 1e-10 || (p2[1] - p1[1]).abs() > 1e-10 {
                            ui.line(Line::new(vec![p1, p2]).color(stroke_color).width(0.5));
                        }
                    }
                });
            } else {
                let (_sy, cy) = self.rotation_y.sin_cos();
                data.faces().iter().for_each(|face| {
                    let center_z = face.0.iter().map(|i| data.vertices()[*i].point().z).sum::<f32>() / face.0.len() as f32;
                    if self.slice_enabled && center_z < self.slice_z { return; }
                    
                    let points: Vec<[f64; 2]> = face.0.iter().map(|&i| projected_vertices[i]).collect();
                    
                    // Guard: skip degenerate (zero-area) polygons in the 2D plot.
                    // egui_plot's internal hit-testing will panic if the polygon's 
                    // bounding box is None (which happens for zero-area polygons).
                    let edge1_x = points[1][0] - points[0][0];
                    let edge1_y = points[1][1] - points[0][1];
                    let edge2_x = points[2][0] - points[0][0];
                    let edge2_y = points[2][1] - points[0][1];
                    let normal_z = edge1_x * edge2_y - edge1_y * edge2_x;
                    
                    if normal_z.abs() < 1e-10 {
                        return;
                    }
                    
                    // Rotate z by current view angle to get view-space depth
                    let view_z = center_z * cy; 
                    let view_z_f = if view_z.is_finite() { view_z as f32 } else { 0.0 };
                    let depth_factor = (view_z_f * 0.5 + 0.5).clamp(0.4, 1.0);
                    
                    let base_color = Color32::from_rgba_unmultiplied(100, 100, 255, 60);
                    let [r, g, b, a] = base_color.to_array();
                    let shaded_color = Color32::from_rgba_unmultiplied(
                        (r as f32 * depth_factor) as u8,
                        (g as f32 * depth_factor) as u8,
                        (b as f32 * depth_factor) as u8,
                        a,
                    );
                    
                    ui.polygon(Polygon::new(points)
                        .stroke(egui::Stroke::new(0.5, stroke_color.linear_multiply(depth_factor)))
                        .fill_color(shaded_color));
                });
            }
        }

        if self.show_interior_points {
            let (_sy, cy) = self.rotation_y.sin_cos();
            data.vertices().iter().enumerate()
                .filter(|(_i, v)| {
                    if self.slice_enabled && v.point().z < self.slice_z { return false; }
                    true
                })
                .for_each(|(i, v)| {
                    let p = projected_vertices[i];
                    let view_z = v.point().z * cy;
                    let view_z_f = if view_z.is_finite() { view_z as f32 } else { 0.0 };
                    let depth_factor = (view_z_f * 0.5 + 0.5).clamp(0.4, 1.0);
                    
                    let base_color = Color32::from_rgb(255, 165, 0);
                    let [r, g, b, _] = base_color.to_array();
                    let shaded_color = Color32::from_rgb(
                        (r as f32 * depth_factor) as u8,
                        (g as f32 * depth_factor) as u8,
                        (b as f32 * depth_factor) as u8,
                    );
                    
                    if p[0].is_finite() && p[1].is_finite() {
                        ui.points(Points::new(vec![p])
                            .radius(1.5)
                            .color(shaded_color));
                    }
                });
        }

        if self.show_constraints {
            // Draw constraint lines
            mesh.constraints().iter().for_each(|(_, constraint)| {
                match constraint {
                    Constraint::Line(arr) => {
                        if self.slice_enabled && (arr[0].z as f32) < self.slice_z && (arr[1].z as f32) < self.slice_z {
                            return;
                        }
                        let p1 = projector.project([arr[0].x as f32, arr[0].y as f32, arr[0].z as f32]);
                        let p2 = projector.project([arr[1].x as f32, arr[1].y as f32, arr[1].z as f32]);
                        ui.line(Line::new(vec![[p1[0], p1[1]], [p2[0], p2[1]]]).color(Color32::GREEN).width(1.2));
                    }
                    Constraint::PolyLine(points) => {
                        let projected: Vec<[f64; 2]> = points.iter()
                            .filter(|p| !self.slice_enabled || (p[0].z as f32) >= self.slice_z)
                            .map(|p| {
                                let proj = projector.project([p[0].x as f32, p[0].y as f32, p[0].z as f32]);
                                [proj[0], proj[1]]
                            })
                            .collect();
                        if !projected.is_empty() {
                            ui.line(Line::new(projected).color(Color32::GREEN).width(1.2));
                        }
                    }
                }
            });
        }
    }

    fn add_bottom_panel(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        let state = self.state_receiver.data.clone();
        let is_generating = matches!(state, State::GeneratingMesh(_));

        let glass_fill = if ui.visuals().dark_mode {
            egui::Color32::from_rgba_unmultiplied(18, 18, 26, 220)
        } else {
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 180)
        };

        ui.add_space(5.0);
        egui::Frame::none()
            .fill(glass_fill)
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(99, 102, 241, 30)))
            .rounding(10.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    super::premium::status_dot(ui, is_generating);
                    ui.add_space(4.0);
                    
                    match &state {
                        State::Idle => {
                            ui.label("Waiting for configuration...");
                            if ui.button(format!("{} Generate mesh", unicode_symbols::REFRESH)).clicked() {
                                self.dialog_state = Some(dialog::State::default());
                            }
                        }
                        State::GeneratingMesh(progress) => {
                            ui.label(format!("Generating mesh: {progress:?}..."));
                        }
                        State::Mesh(_) => {
                            self.add_regen_button_and_viewmode_toggles(ui);
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let resp = ui.add(
                                    egui::Button::new("Next: Physics & Simulation ➡")
                                        .fill(ui.visuals().selection.bg_fill)
                                        .rounding(Rounding::same(6.0))
                                ).on_hover_text("Proceed to configure physics properties and run the simulation");
                                if resp.clicked() {
                                    self.run_simulation_clicked = true;
                                }
                            });
                        }
                    }
                });
            });

        if let State::Mesh(mesh) = state {
            Self::mesh_info(ui, mesh.triangulation_data());
        }
    }

    fn add_regen_button_and_viewmode_toggles(&mut self, ui: &mut Ui) {
        let response = ui
            .button(format!("{} Regenerate mesh", unicode_symbols::REFRESH))
            .on_hover_text("Click to regenerate mesh");
        if response.clicked() {
            self.dialog_state = Some(dialog::State::default());
        }
        if ui.button(format!("{} Reset view", unicode_symbols::REFRESH)).clicked() {
            self.rotation_x = -0.5;
            self.rotation_y = 0.5;
            self.needs_reset = true;
        }
        ui.checkbox(&mut self.show_constraints, "Show constraints");
        ui.checkbox(&mut self.show_interior_points, "Show particles");
        if !self.hide_mesh {
            ui.checkbox(&mut self.show_wireframe_only, "Show wireframe only");
            ui.checkbox(&mut self.slice_enabled, "Enable Z-Slice");
            if self.slice_enabled {
                ui.add(egui::Slider::new(&mut self.slice_z, -20.0..=20.0).text("Z"));
            }
        }
        ui.checkbox(&mut self.auto_rotate, "Auto-Rotate 360°");
        ui.checkbox(&mut self.hide_mesh, "Hide mesh");
    }

    fn mesh_info(ui: &mut Ui, triangulation_data: &triangulation::Data) {
        ui.horizontal(|ui| {
            ui.label(format!("Elements: {}", triangulation_data.faces().len()));
            ui.label(format!("Points: {}", triangulation_data.vertices().len()));
        });
    }

    #[must_use]
    fn input_dialog_and_error_ui(
        ui: &mut Ui,
        dialog_state: &mut Option<dialog::State>,
        input_error: &mut Option<String>,
        solid_bounds: Option<([f64; 3], [f64; 3])>,
    ) -> Option<dialog::Data> {
        if let Some(err) = input_error.as_ref() {
            if error_dialog::show(err, ui.ctx()).closed() {
                *input_error = None;
            }
            return None;
        }

        let mut state = dialog_state.take()?;
        use dialog::Response;
        match dialog::show(&mut state, ui.ctx(), solid_bounds) {
            Response::Noop => {
                *dialog_state = Some(state);
                None
            }
            Response::DataResult(result) => match result {
                Ok(data) => Some(data),
                Err(err) => {
                    *dialog_state = Some(state);
                    *input_error = err.into();
                    None
                }
            },
            Response::Cancel => None,
        }
    }
    fn add_physics_preview_sidebar(&mut self, ui: &mut Ui) {
        puffin::profile_function!();

        // Fixed width: animating the panel width changes the central plot rect every frame.
        // With `data_aspect(1.0)`, egui_plot then adjusts bounds each frame (`set_aspect_by_changing_axis`),
        // which reads as unwanted zoom; combined with a resizing plot, pointer deltas can look like drags
        // and spin the 3D view.
        const SIDEBAR_WIDTH: f32 = 280.0;

        let glass_fill = if ui.visuals().dark_mode {
            egui::Color32::from_rgba_unmultiplied(14, 14, 22, 215)
        } else {
            egui::Color32::from_rgba_unmultiplied(250, 252, 255, 220)
        };

        egui::SidePanel::left("meshing_physics_preview")
            .resizable(true)
            .default_width(SIDEBAR_WIDTH)
            .min_width(200.0)
            .frame(egui::Frame::none()
                .fill(glass_fill)
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(99, 102, 241, 25)))
                .inner_margin(8.0))
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(8.0);
                    super::premium::premium_card(ui, "🧬 Physics & Material", |ui| {
                        ui.vertical(|ui| {
                            ui.label("Current state: Mesh Generation.");
                            ui.add_space(10.0);
                            ui.label("ℹ Material properties (Steel, Concrete, etc.) are configured in the next 'Simulation' step.");
                            ui.add_space(10.0);
                            ui.label("The object will then be simulated using those physical properties.");
                        });
                    });

                    ui.add_space(20.0);
                    super::premium::premium_card(ui, "🎮 Viewport", |ui| {
                        ui.vertical_centered_justified(|ui| {
                            if ui.button("⟳ Reset Camera View").clicked() {
                                self.rotation_x = -0.5;
                                self.rotation_y = 0.5;
                                self.needs_reset = true;
                            }
                        });
                        ui.add_space(4.0);
                        ui.vertical(|ui| {
                            ui.small("🖱 Rotate: Right-click + Drag");
                            ui.small("🔍 Zoom: Ctrl + Scroll");
                            ui.small("✋ Pan: Left-click + Drag");
                        });
                    });
                });
            });
    }
}

mod serde_impl {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Debug, Serialize, Deserialize)]
    enum WrappedProjectData {
        WithBoundaryConditions(Data<WithBoundaryConditions>),
        WithMesh(Data<WithMesh>),
    }

    impl Serialize for Page {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let project_data = self.mesh_generator.project_data().clone();
            let project_data = match &self.state_receiver.data {
                State::Mesh(mesh) => {
                    WrappedProjectData::WithMesh(project_data.with_mesh(mesh.clone()))
                }
                _ => WrappedProjectData::WithBoundaryConditions(project_data),
            };
            project_data.serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Page {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            WrappedProjectData::deserialize(deserializer).map(|wpd| match wpd {
                WrappedProjectData::WithBoundaryConditions(project_data) => {
                    Page::from(project_data)
                }
                WrappedProjectData::WithMesh(project_data) => Page::from(project_data),
            })
        }
    }
}
