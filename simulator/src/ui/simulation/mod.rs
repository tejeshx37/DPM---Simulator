mod config_dialog;
mod plot_dialog;

// ── 4.5 Colormap selection ───────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Colormap {
    #[default]
    CoolWarm,  // diverging blue→white→red (default, classic FEM)
    Viridis,   // perceptually uniform purple→yellow
    Plasma,    // purple→orange→yellow
    Jet,       // blue→cyan→green→yellow→red
    Grayscale, // black→white
}

impl Colormap {
    fn label(self) -> &'static str {
        match self {
            Colormap::CoolWarm  => "Cool-Warm",
            Colormap::Viridis   => "Viridis",
            Colormap::Plasma    => "Plasma",
            Colormap::Jet       => "Jet",
            Colormap::Grayscale => "Grayscale",
        }
    }
    /// Map t ∈ [0,1] (0=min stress, 1=max stress) → Color32
    fn map(self, t: f32) -> egui::Color32 {
        let t = if t.is_finite() { t } else { 0.0 };
        let t = t.clamp(0.0, 1.0);
        match self {
            Colormap::CoolWarm => {
                // blue(0)→white(0.5)→red(1)
                let (r, g, b) = if t < 0.5 {
                    let s = t * 2.0;
                    (lerp(59, 255, s), lerp(76, 255, s), lerp(192, 255, s))
                } else {
                    let s = (t - 0.5) * 2.0;
                    (lerp(255, 180, s), lerp(255, 4, s), lerp(255, 38, s))
                };
                egui::Color32::from_rgb(r, g, b)
            }
            Colormap::Viridis => {
                // sampled 8-stop LUT
                const LUT: [(u8,u8,u8); 8] = [
                    (68,1,84),(72,40,120),(62,83,164),(49,120,150),
                    (53,183,121),(109,205,89),(180,222,44),(253,231,37),
                ];
                lut_sample(&LUT, t)
            }
            Colormap::Plasma => {
                const LUT: [(u8,u8,u8); 8] = [
                    (13,8,135),(75,3,161),(156,23,158),(210,51,100),
                    (240,97,45),(252,157,25),(242,216,40),(240,249,33),
                ];
                lut_sample(&LUT, t)
            }
            Colormap::Jet => {
                // blue→cyan→green→yellow→red
                let (r, g, b) = if t < 0.25 {
                    let s = t * 4.0;
                    (0, lerp(0, 255, s), 255)
                } else if t < 0.5 {
                    let s = (t - 0.25) * 4.0;
                    (0, 255, lerp(255, 0, s))
                } else if t < 0.75 {
                    let s = (t - 0.5) * 4.0;
                    (lerp(0, 255, s), 255, 0)
                } else {
                    let s = (t - 0.75) * 4.0;
                    (255, lerp(255, 0, s), 0)
                };
                egui::Color32::from_rgb(r, g, b)
            }
            Colormap::Grayscale => {
                let v = (t * 255.0) as u8;
                egui::Color32::from_rgb(v, v, v)
            }
        }
    }
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t) as u8
}
fn lut_sample(lut: &[(u8,u8,u8)], t: f32) -> egui::Color32 {
    let n = lut.len();
    let idx = (t * (n - 1) as f32).floor() as usize;
    let idx = idx.min(n - 2);
    let frac = t * (n - 1) as f32 - idx as f32;
    let (r0,g0,b0) = lut[idx];
    let (r1,g1,b1) = lut[idx+1];
    egui::Color32::from_rgb(
        lerp(r0, r1, frac), lerp(g0, g1, frac), lerp(b0, b1, frac)
    )
}

// ── 4.3 Slice axis ───────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum SliceAxis { X, Y, #[default] Z }

// ── 4.6 Overlay mode ─────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum OverlayMode {
    #[default]
    Stress,        // colour by selected stress component
    Displacement,  // colour by ‖x − x₀‖
    VonMises,      // colour by von Mises stress
}

#[derive(Debug, Clone, Copy)]
struct AnalyticsPoint {
    time: f64,
    total_strain_energy: f64,
    total_kinetic_energy: f64,
    max_stress: f64,
    broken_count: u64,
    #[allow(dead_code)]
    inverted_count: u64,
    max_displacement: f64,
    centre_of_mass_drift: f64,
}


use super::{bottom_panel, error_dialog, gnomon, plot_utils, unicode_symbols, ContextWrapper};
use crate::model::{
    engine::{Config, Engine, Frame, Senders, State},
    project::data::{Data, WithBoundaryConditions, WithCpdExportData, WithMesh, WithShape},
    state_channel::{self, Receiver, STReceiver},
};
use cgal::BoundaryId;
use cpd::{BoundaryAverage, ExportData, TimeStampedValue};
use egui::{
    Button, CentralPanel, CollapsingHeader, Color32, Key, Modifiers, SidePanel,
    Stroke, Ui, Vec2, Vec2b, WidgetText,
};
use egui_plot::{AxisHints, Line, Plot, PlotPoint, PlotUi, Polygon};
use nalgebra::{Matrix3, Vector3};
use nalgebra_ext::matrix3::Component;
use rayon::prelude::*;
use std::hash::Hash;
use strum::IntoEnumIterator;
use typed_builder::TypedBuilder;

const ORANGE: Color32 = Color32::from_rgb(0xFF, 0xA5, 0x00);

#[derive(Debug, TypedBuilder)]
struct Receivers {
    config_receiver: Receiver<Box<Config>, Option<Box<Config>>>,
    state_receiver: STReceiver<State>,
    frame_receiver: Receiver<Frame, Option<Frame>>,
    error_receiver: Receiver<String, Option<String>>,
}

#[derive(Debug)]
pub struct Page {
    engine: Engine<ContextWrapper>,
    config_dialog_state: Option<config_dialog::State>,
    configure_error: Option<String>,
    show_stress_gradients: bool,
    stress_tensor_component: Component,
    receivers: Receivers,
    selected_element_index: Option<usize>,
    selected_vertex_index: Option<usize>,
    plot_dialog_state: Option<plot_dialog::State>,
    selected_boundary_id: Option<BoundaryId>,
    rotation_x: f32,
    rotation_y: f32,
    slice_enabled: bool,
    slice_axis: SliceAxis,
    slice_offset: f32,
    analytics_history: Vec<AnalyticsPoint>,
    show_analytics: bool,
    show_stress_plot: bool,
    gpu_pipeline: Option<std::sync::Arc<cpd_wgpu::ComputePipeline>>,
    show_broken_elements: bool,
    show_force_vectors: bool,
    auto_rotate: bool,
    // 4.5 Colormap
    colormap: Colormap,
    // 4.6 Overlay mode
    overlay_mode: OverlayMode,
    // 4.7 Playback speed
    playback_speed: f32,
    #[allow(dead_code)]
    steps_accumulator: f32,
    engine_alive: bool,
}

#[derive(Debug)]
pub enum MenuResponse {
    Noop(Page),
    EditMesh(Data<WithMesh>),
    EditBoundaryConditions(Data<WithBoundaryConditions>),
    EditShape(Data<WithShape>),
}

#[derive(Debug)]
pub enum Response {
    Noop(Page),
}

fn senders_and_receivers() -> (Senders, Receivers) {
    let (config_sender, config_receiver) = state_channel::with_default(1);
    let (state_sender, state_receiver) = state_channel::same_type_with_default(1);
    let (frame_sender, frame_receiver) = state_channel::with_default(1);
    let (error_sender, error_receiver) = state_channel::with_default(1);
    let senders = Senders::builder()
        .config_sender(config_sender)
        .state_sender(state_sender)
        .frame_sender(frame_sender)
        .error_sender(error_sender)
        .build();
    let receivers = Receivers::builder()
        .config_receiver(config_receiver)
        .state_receiver(state_receiver)
        .frame_receiver(frame_receiver)
        .error_receiver(error_receiver)
        .build();
    (senders, receivers)
}

impl Page {
    pub fn new(project_data: Data<WithMesh>, gpu_pipeline: Option<std::sync::Arc<cpd_wgpu::ComputePipeline>>) -> Self {
        let (senders, receivers) = senders_and_receivers();
        Self::with_engine(Engine::new(project_data, senders, gpu_pipeline.clone()), receivers, gpu_pipeline)
    }

    pub fn try_new(project_data: Data<WithCpdExportData>, gpu_pipeline: Option<std::sync::Arc<cpd_wgpu::ComputePipeline>>) -> Result<Self, String> {
        let (senders, receivers) = senders_and_receivers();
        Engine::new_with_cpd_data(project_data, senders, gpu_pipeline.clone())
            .map(|engine| Self::with_engine(engine, receivers, gpu_pipeline))
    }
}

impl From<Data<WithMesh>> for Page {
    fn from(project_data: Data<WithMesh>) -> Self {
        Self::new(project_data, None)
    }
}

impl TryFrom<Data<WithCpdExportData>> for Page {
    type Error = String;

    fn try_from(project_data: Data<WithCpdExportData>) -> Result<Self, Self::Error> {
        Self::try_new(project_data, None)
    }
}

#[derive(Debug, Default, Clone, Copy)]
enum FramePlotHoverResponse {
    #[default]
    Noop,
    ElementIndex(usize),
    #[allow(dead_code)]
    VertexIndex(usize),
}


#[derive(Debug, Default, Clone, Copy)]
enum FramePreviewResponse {
    #[default]
    Noop,
    ElementSelected(usize),
    VertexSelected(usize),
}

use super::plot_utils::Projector;

impl Page {
    fn with_engine(engine: Engine<ContextWrapper>, receivers: Receivers, gpu_pipeline: Option<std::sync::Arc<cpd_wgpu::ComputePipeline>>) -> Self {
        Self {
            engine,
            config_dialog_state: None,
            configure_error: None,
            show_stress_gradients: false,
            stress_tensor_component: Component::default(),
            receivers,
            selected_element_index: None,
            selected_vertex_index: None,
            plot_dialog_state: None,
            selected_boundary_id: None,
            rotation_x: 0.0,
            rotation_y: 0.0,
            slice_enabled: false,
            slice_axis: SliceAxis::Z,
            slice_offset: 0.0,
            analytics_history: Vec::new(),
            show_analytics: false,
            show_stress_plot: true,
            gpu_pipeline,
            show_broken_elements: true,
            show_force_vectors: false,
            auto_rotate: false,
            colormap: Colormap::CoolWarm,
            overlay_mode: OverlayMode::Stress,
            playback_speed: 1.0,
            steps_accumulator: 0.0,
            engine_alive: true,
        }
    }

    fn add_edit_menu(self, ui: &mut Ui) -> MenuResponse {
        puffin::profile_function!();
        #[derive(Debug, Default)]
        struct Response {
            edit_mesh: bool,
            edit_bc: bool,
            edit_shape: bool,
        }
        let opt = ui
            .menu_button("Edit", |ui| {
                let mut response = Response::default();
                if ui.button("Edit mesh").clicked() {
                    response.edit_mesh = true;
                    ui.close_menu();
                }
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
        if response.edit_mesh {
            MenuResponse::EditMesh(self.engine.take_project_data())
        } else if response.edit_bc {
            MenuResponse::EditBoundaryConditions(self.engine.take_project_data().without_mesh().0)
        } else if response.edit_shape {
            MenuResponse::EditShape(
                self.engine
                    .take_project_data()
                    .without_mesh()
                    .0
                    .without_boundary_conditions()
                    .0,
            )
        } else {
            MenuResponse::Noop(self)
        }
    }

    fn add_plot_menu(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        ui.menu_button("Plot", |ui| {
            let discard_plot =
                self.selected_element_index.is_some() && ui.button("Discard stress plot").clicked();
            if discard_plot {
                ui.close_menu();
                self.stop_recording_stress_data();
            }

            let discard_plot = self.selected_vertex_index.is_some()
                && ui.button("Discard displacement plot").clicked();
            if discard_plot {
                ui.close_menu();
                self.stop_recording_vertex_position();
            }

            let response = ui.button("Boundary average").on_hover_text(
                "Plot average displacement or force on a boundary. \n\
            Click to open a dialog from where you can choose the boundary.",
            );
            if response.clicked() {
                ui.close_menu();
                self.plot_dialog_state = Some(plot_dialog::State::new(
                    self.engine.plot_items().clone(),
                    self.engine.polygon_data().clone(),
                ));
            }

            let discard_plot = self.selected_boundary_id.is_some()
                && ui.button("Discard boundary average plot").clicked();
            if discard_plot {
                ui.close_menu();
                self.selected_boundary_id = None;
                self.engine.stop_recording_boundary_data();
            }
        });
    }

    #[must_use]
    pub fn add_menu_items(self, ui: &mut Ui) -> MenuResponse {
        puffin::profile_function!();
        let response = self.add_edit_menu(ui);
        let MenuResponse::Noop(mut page) = response else {
            return response;
        };
        page.add_plot_menu(ui);
        MenuResponse::Noop(page)
    }

    fn color_for_component(component: Component) -> Color32 {
        match component {
            Component::XX => Color32::RED,
            Component::XY => ORANGE,
            Component::XZ => Color32::LIGHT_BLUE,
            Component::YX => Color32::YELLOW,
            Component::YY => Color32::LIGHT_GREEN,
            Component::YZ => Color32::BROWN,
            Component::ZX => Color32::BLUE,
            Component::ZY => Color32::KHAKI,
            Component::ZZ => Color32::GREEN,
        }
    }

    fn instructions(ui: &mut Ui, state: &State) {
        puffin::profile_function!();
        egui::CollapsingHeader::new("ℹ️ Simulation Guide")
            .default_open(false)
            .show(ui, |ui| {
                ui.add_space(4.0);
                if state == &State::Unconfigured {
                    ui.label(egui::RichText::new("⚠️ System Unconfigured").color(Color32::from_rgb(251, 191, 36)).strong());
                    ui.label("Please configure the physics engine in the sidebar to begin.");
                    ui.add_space(8.0);
                }

                ui.scope(|ui| {
                    ui.spacing_mut().item_spacing.y = 8.0;
                    
                    ui.label(egui::RichText::new("🖱 Interaction").strong());
                    ui.horizontal(|ui| {
                        ui.label("•");
                        ui.label("Left-click an element to track its stress history.");
                    });
                    ui.horizontal(|ui| {
                        ui.label("•");
                        ui.label("Right-click a node to track its displacement.");
                    });
                    ui.horizontal(|ui| {
                        ui.label("•");
                        ui.label("Right-click + Drag to rotate the 3D view.");
                    });
                    
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("📊 Analytics").strong());
                    ui.label("Use the 'Plot' menu to track boundary averages and manage active plots.");
                    
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("⚙️ Performance").strong());
                    ui.label("Disable 'Stress Gradients' in the sidebar for faster real-time processing.");
                });
                ui.add_space(8.0);
            });
    }

    #[must_use]
    pub fn add_contents(mut self, ui: &mut Ui) -> Response {
        puffin::profile_function!();
        if self.engine_alive {
            self.engine.set_refresh_token(ui.ctx());
        }
        ui.heading("Simulation");

        // --- Auto-rotation Logic ---
        if self.auto_rotate && self.receivers.state_receiver.data == State::Running {
            self.rotation_y += ui.input(|i| i.stable_dt) * 0.4;
            ui.ctx().request_repaint(); // Keep animating
        }

        macro_rules! update_receivers {
            ( $( $receiver:ident ),*) => {
                $(
                    if let Err(_) = self.receivers.$receiver.update() {
                         self.engine_alive = false;
                    }
                )*
            };
        }
        update_receivers!(
            frame_receiver,
            state_receiver,
            config_receiver,
            error_receiver
        );

        if let Some(frame) = self.receivers.frame_receiver.data.as_ref() {
            let data = frame.data();
            let config = self.receivers.config_receiver.data.as_ref();
            if let Some(config) = config {
                let dt = *config.cpd_config().time_delta();
                let time = *frame.iterations() as f64 * dt.as_secs_f64();
                
                // Only add if time is newer
                if self.analytics_history.last().map(|p| p.time < time).unwrap_or(true) {
                    let nodes = data.nodes();
                    let elements = data.elements();

                    let total_strain_energy: f32 = elements.par_iter()
                        .map(|e| e.strain_energy())
                        .sum();

                    let total_kinetic_energy: f32 = nodes.par_iter()
                        .map(|n| 0.5 * n.mass() * n.velocity().norm_squared())
                        .sum();

                    let broken_count = elements.par_iter()
                        .filter(|e| *e.is_broken())
                        .count() as u64;

                    let inverted_count = elements.par_iter()
                        .filter(|e| *e.is_inverted())
                        .count() as u64;

                    // Max displacement: ‖current_pos − initial_pos‖ over all nodes
                    let max_displacement: f32 = nodes.par_iter()
                        .map(|n| (n.position() - n.initial_position()).norm())
                        .reduce(|| 0.0f32, f32::max);

                    // Centre-of-mass drift: ‖CoM_current − CoM_initial‖
                    let total_mass: f32 = nodes.par_iter().map(|n| n.mass()).sum::<f32>().max(1e-12);
                    let com_current: Vector3<f32> = nodes.par_iter()
                        .map(|n| *n.position() * n.mass())
                        .reduce(Vector3::zeros, |a, b| a + b)
                        / total_mass;
                    let com_initial: Vector3<f32> = nodes.par_iter()
                        .map(|n| *n.initial_position() * n.mass())
                        .reduce(Vector3::zeros, |a, b| a + b)
                        / total_mass;
                    let centre_of_mass_drift = (com_current - com_initial).norm();

                    let max_stress = *data.max_stress().index(self.stress_tensor_component);

                    self.analytics_history.push(AnalyticsPoint {
                        time,
                        total_strain_energy: total_strain_energy as f64,
                        total_kinetic_energy: total_kinetic_energy as f64,
                        max_stress: max_stress as f64,
                        broken_count,
                        inverted_count,
                        max_displacement: max_displacement as f64,
                        centre_of_mass_drift: centre_of_mass_drift as f64,
                    });

                    // Keep last 1000 points for performance
                    if self.analytics_history.len() > 1000 {
                        self.analytics_history.remove(0);
                    }
                }
            }
        }

        Self::instructions(ui, &self.receivers.state_receiver.data);

        bottom_panel::show("simulation_bottom_panel", ui, |ui| self.add_controls(ui));

        self.add_physics_sidebar(ui);

        macro_rules! frame {
            () => {
                self.receivers.frame_receiver.data.as_ref()
            };
        }

        let element_opt = self
            .selected_element_index
            .and_then(|index| frame!().map(|frame| &frame.data().elements()[index]))
            .and_then(|element| element.stress_time_series().as_series());

        if self.show_stress_plot {
            if let Some(series) = element_opt {
                SidePanel::left("simulation_left_plot_panel")
                    .show_inside(ui, |ui| self.show_time_series_stress_plot(ui, series));
            }
        }

        macro_rules! right_plot_panel {
            ( $add_contents:expr ) => {
                SidePanel::right("simulation_right_plot_panel").show_inside(ui, $add_contents);
            };
        }
        let node_opt = self
            .selected_vertex_index
            .and_then(|index| frame!().map(|frame| &frame.data().nodes()[index]))
            .and_then(|node| node.position_time_series().as_series());
        let boundary_avg_opt =
            frame!().and_then(|frame| frame.data().boundary_average_data().as_ref());
        match (node_opt, boundary_avg_opt) {
            (None, None) => {}
            (None, Some(data)) => {
                right_plot_panel!(|ui| {
                    self.show_time_series_boundary_average_data_plot(ui, data);
                });
            }
            (Some(series), None) => {
                right_plot_panel!(|ui| { self.show_time_series_displacement_plot(ui, series) });
            }
            (Some(series), Some(data)) => {
                right_plot_panel!(|ui| {
                    let size = ui.available_size();
                    let avg_plot_size = Vec2::new(size.x, size.y * 2.0 / 3.0);
                    ui.allocate_ui(avg_plot_size, |ui| {
                        self.show_time_series_boundary_average_data_plot(ui, data)
                    });
                    ui.separator();
                    self.show_time_series_displacement_plot(ui, series);
                });
            }
        }

        if self.show_analytics && !self.analytics_history.is_empty() {
            // Smooth slide-in: animate the panel width from 0→300 over 0.35s
            let panel_anim = ui.ctx().animate_bool_with_time(
                egui::Id::new("analytics_panel_anim"),
                true,
                0.35,
            );
            let panel_width = egui::lerp(0.0..=320.0, panel_anim);

            let glass_fill = if ui.visuals().dark_mode {
                egui::Color32::from_rgba_unmultiplied(16, 16, 24, 210)
            } else {
                egui::Color32::from_rgba_unmultiplied(248, 250, 255, 220)
            };

            SidePanel::right("simulation_analytics_panel")
                .resizable(true)
                .default_width(panel_width)
                .min_width(panel_width.min(320.0))
                .frame(egui::Frame::none()
                    .fill(glass_fill)
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(99, 102, 241, 30)))
                    .inner_margin(8.0))
                .show_inside(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.add_space(8.0);

                        // ── Plot: Total Strain Energy ─────────────────────────
                        super::premium::premium_card(ui, "📊 Real-Time Analytics", |ui| {
                            ui.label(egui::RichText::new("Total Strain Energy (J)").small().strong());
                            Plot::new("energy_plot")
                                .height(120.0)
                                .allow_drag(false).allow_zoom(false).allow_scroll(false)
                                .show_grid(false)
                                .show_axes(false)
                                .show(ui, |plot_ui| {
                                    let pts: Vec<[f64; 2]> = self.analytics_history.iter()
                                        .map(|p| [p.time, p.total_strain_energy])
                                        .filter(|p| p[0].is_finite() && p[1].is_finite())
                                        .collect();
                                    plot_ui.line(Line::new(pts)
                                        .color(Color32::from_rgb(129, 140, 248)).width(2.0));
                                });

                            ui.add_space(10.0);

                            // ── Plot: Total Kinetic Energy ────────────────────
                            ui.label(egui::RichText::new("Total Kinetic Energy (J)").small().strong());
                            Plot::new("kinetic_energy_plot")
                                .height(120.0)
                                .allow_drag(false).allow_zoom(false).allow_scroll(false)
                                .show_grid(false)
                                .show_axes(false)
                                .show(ui, |plot_ui| {
                                    let pts: Vec<[f64; 2]> = self.analytics_history.iter()
                                        .map(|p| [p.time, p.total_kinetic_energy])
                                        .filter(|p| p[0].is_finite() && p[1].is_finite())
                                        .collect();
                                    plot_ui.line(Line::new(pts)
                                        .color(Color32::from_rgb(52, 211, 153)).width(2.0));
                                });

                            ui.add_space(10.0);

                            // ── Plot: Peak Stress ─────────────────────────────
                            ui.label(egui::RichText::new(
                                format!("Peak Stress ({}) (Pa)", self.stress_tensor_component)
                            ).small().strong());
                            Plot::new("max_stress_plot")
                                .height(120.0)
                                .allow_drag(false).allow_zoom(false).allow_scroll(false)
                                .show_grid(false)
                                .show_axes(false)
                                .show(ui, |plot_ui| {
                                    let pts: Vec<[f64; 2]> = self.analytics_history.iter()
                                        .map(|p| [p.time, p.max_stress])
                                        .filter(|p| p[0].is_finite() && p[1].is_finite())
                                        .collect();
                                    plot_ui.line(Line::new(pts)
                                        .color(Color32::from_rgb(248, 113, 113)).width(2.0));
                                });

                            ui.add_space(10.0);

                            // ── Plot: Broken Element Count ────────────────────
                            ui.label(egui::RichText::new("Fractured Elements").small().strong());
                            Plot::new("broken_count_plot")
                                .height(100.0)
                                .allow_drag(false).allow_zoom(false).allow_scroll(false)
                                .show_grid(false)
                                .show_axes(false)
                                .show(ui, |plot_ui| {
                                    let pts: Vec<[f64; 2]> = self.analytics_history.iter()
                                        .map(|p| [p.time, p.broken_count as f64])
                                        .filter(|p| p[0].is_finite() && p[1].is_finite())
                                        .collect();
                                    plot_ui.line(Line::new(pts)
                                        .color(Color32::from_rgb(251, 146, 60)).width(2.0));
                                });
                        });

                        ui.add_space(12.0);

                        // ── Metrics Summary Card ──────────────────────────────
                        super::premium::premium_card(ui, "📈 Metrics Summary", |ui| {
                            ui.add_space(4.0);

                            if let Some(latest) = self.analytics_history.last() {
                                let row = |ui: &mut egui::Ui, label: &str, value: String| {
                                    ui.horizontal(|ui| {
                                        ui.label(label);
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| { ui.strong(value); },
                                        );
                                    });
                                };

                                row(ui, "⏱ Elapsed Time:",
                                    format!("{:.4} s", latest.time));
                                row(ui, "🔋 Strain Energy:",
                                    format!("{:.3e} J", latest.total_strain_energy));
                                row(ui, "⚡ Kinetic Energy:",
                                    format!("{:.3e} J", latest.total_kinetic_energy));

                                // Total mechanical energy
                                let total_energy =
                                    latest.total_strain_energy + latest.total_kinetic_energy;
                                row(ui, "∑ Total Energy:",
                                    format!("{:.3e} J", total_energy));

                                ui.separator();

                                row(ui, "📐 Peak Stress (σ):",
                                    Self::format_stress(latest.max_stress as f32));
                                row(ui, "💥 Fractured Elements:",
                                    format!("{}", latest.broken_count));
                                row(ui, "📏 Max Displacement:",
                                    format!("{:.4e} m", latest.max_displacement));
                                row(ui, "🎯 CoM Drift:",
                                    format!("{:.4e} m", latest.centre_of_mass_drift));
                            }

                            ui.add_space(10.0);
                            ui.separator();
                            ui.add_space(8.0);

                            ui.horizontal(|ui| {
                                if ui.button("⬇ Export CSV").clicked() {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .set_file_name("simulation_analytics.csv")
                                        .add_filter("CSV", &["csv"])
                                        .save_file()
                                    {
                                        let mut csv = String::from(
                                            "time,strain_energy,kinetic_energy,total_energy,\
                                             max_stress,broken_count,max_displacement,com_drift\n"
                                        );
                                        for pt in &self.analytics_history {
                                            csv.push_str(&format!(
                                                "{:.6},{:.6},{:.6},{:.6},{:.6},{},{:.6},{:.6}\n",
                                                pt.time,
                                                pt.total_strain_energy,
                                                pt.total_kinetic_energy,
                                                pt.total_strain_energy + pt.total_kinetic_energy,
                                                pt.max_stress,
                                                pt.broken_count,
                                                pt.max_displacement,
                                                pt.centre_of_mass_drift,
                                            ));
                                        }
                                        let _ = std::fs::write(&path, csv);
                                    }
                                }
                                if ui.button("🗑 Clear").clicked() {
                                    self.analytics_history.clear();
                                }
                            });
                        });
                        ui.add_space(20.0);
                        ui.label("Playback Speed:");
                        ui.add(egui::Slider::new(&mut self.playback_speed, 0.1..=4.0).text("x").logarithmic(true));
                        
                        // Set the speed on the engine if it changed
                        // Playback speed is now handled in the engine update loop

                        ui.add_space(20.0);
                    });
                });
        }

        match self.show_preview(ui) {
            FramePreviewResponse::Noop => {}
            FramePreviewResponse::ElementSelected(handle) => self.record_stress_data(handle),
            FramePreviewResponse::VertexSelected(index) => self.record_vertex_position(index),
        }

        if let Some(err) = &self.receivers.error_receiver.data {
            if error_dialog::show(err, ui.ctx()).closed() {
                self.receivers.error_receiver.data = None;
            }
        }

        let config = Self::input_dialog_and_error_ui(
            ui,
            &mut self.config_dialog_state,
            &mut self.configure_error,
            self.gpu_pipeline.is_some(),
        );
        if let Some(config) = config {
            self.configure_error = None;
            if self.engine_alive {
                self.engine.configure(config);
            }
        }

        if let Some(state) = &mut self.plot_dialog_state {
            use plot_dialog::Response;
            match plot_dialog::show(ui.ctx(), state) {
                Response::Noop => {}
                Response::Cancel => {
                    self.plot_dialog_state = None;
                }
                Response::BoundaryId(id) => {
                    self.plot_dialog_state = None;
                    if self.selected_boundary_id != Some(id) {
                        self.selected_boundary_id = Some(id);
                        if self.engine_alive {
                            self.engine.record_boundary_data(id);
                        }
                    }
                }
            }
        }

        Response::Noop(self)
    }

    fn add_physics_sidebar(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        

        // Smooth slide-in on first render
        let sidebar_anim = ui.ctx().animate_bool_with_time(
            egui::Id::new("physics_sidebar_anim"), true, 0.3
        );
        let sidebar_width = egui::lerp(0.0..=280.0, sidebar_anim);

        let glass_fill = if ui.visuals().dark_mode {
            egui::Color32::from_rgba_unmultiplied(14, 14, 22, 215)
        } else {
            egui::Color32::from_rgba_unmultiplied(250, 252, 255, 220)
        };

        SidePanel::left("simulation_physics_panel")
            .resizable(true)
            .default_width(sidebar_width)
            .min_width(sidebar_width.min(280.0))
            .frame(egui::Frame::none()
                .fill(glass_fill)
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(99, 102, 241, 25)))
                .inner_margin(8.0))
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(8.0);
                    
                    // --- Physics & Material Card ---
                    super::premium::premium_card(ui, "🧬 Physics & Material", |ui| {
                        let _state = self.receivers.state_receiver.data;
                        let config = self.receivers.config_receiver.data.as_ref();

                        if let Some(config) = config {
                            let mp = config.cpd_config().material_props();
                            let bp = mp.bulk_props();

                            ui.horizontal(|ui| {
                                ui.label("Density:");
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.strong(format!("{} kg/m³", bp.density()));
                                });
                            });
                            ui.horizontal(|ui| {
                                ui.label("Damping:");
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.strong(format!("{:.4}", bp.damping()));
                                });
                            });

                            ui.separator();

                            match mp {
                                cpd::MaterialProps::Isotropic(p) => {
                                    ui.label(egui::RichText::new("Isotropic Model").italics().small());
                                    ui.horizontal(|ui| {
                                        ui.label("Modulus (E):");
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.strong(format!("{:.2e} Pa", p.elasticity_modulus()));
                                        });
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Poisson (ν):");
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.strong(format!("{:.2}", p.poissons_ratio()));
                                        });
                                    });
                                }
                                cpd::MaterialProps::Orthotropic(p) => {
                                    ui.label(egui::RichText::new("Orthotropic Model").italics().small());
                                    ui.label(format!("Ex: {:.2e}", p.elasticity_modulus_x()));
                                    ui.label(format!("Ey: {:.2e}", p.elasticity_modulus_y()));
                                }
                            }
                            
                            ui.add_space(12.0);
                            if ui.button(format!("{} Edit Material Properties", unicode_symbols::GEAR)).clicked() {
                                let mesh = self.engine.project_data().state().mesh.clone();
                                self.config_dialog_state.replace(config_dialog::State::new(config, mesh));
                            }
                        } else {
                            ui.vertical_centered(|ui| {
                                ui.add_space(10.0);
                                ui.colored_label(ORANGE, "Engine not configured");
                                if ui.button("🚀 Initialize Solver").clicked() {
                                    let mesh = self.engine.project_data().state().mesh.clone();
                                    self.config_dialog_state.replace(config_dialog::State::default(mesh));
                                }
                                ui.add_space(10.0);
                            });
                        }
                    });

                    ui.add_space(12.0);

                    // --- Simulation Config Card ---
                    if let Some(config) = self.receivers.config_receiver.data.as_ref() {
                        super::premium::premium_card(ui, "⏱ Simulation Control", |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Total Duration:");
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.strong(format!("{:.2} s", config.cpd_config().duration().as_secs_f32()));
                                });
                            });
                            ui.horizontal(|ui| {
                                ui.label("Time Step (Δt):");
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.strong(format!("{:.1e} s", config.cpd_config().time_delta().as_secs_f64()));
                                });
                            });
                            ui.horizontal(|ui| {
                                ui.label("Refresh Sync:");
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(format!("{} steps", config.refresh_period()));
                                });
                            });
                        });
                        ui.add_space(12.0);
                    }

                    // ── Visualization Settings ─────────────────────────────
                    super::premium::premium_card(ui, "👁 Visualization", |ui| {
                        ui.checkbox(&mut self.show_analytics, "Real-time Analytics HUD");
                        ui.checkbox(&mut self.show_stress_plot, "Show Stress Plot");
                        ui.checkbox(&mut self.auto_rotate, "Continuous Auto-Rotation");

                        ui.separator();

                        // 4.6 Overlay mode
                        ui.label(egui::RichText::new("Colour Overlay").small().strong());
                        ui.horizontal(|ui| {
                            ui.radio_value(&mut self.overlay_mode, OverlayMode::Stress,       "Stress");
                            ui.radio_value(&mut self.overlay_mode, OverlayMode::VonMises,     "von Mises");
                            ui.radio_value(&mut self.overlay_mode, OverlayMode::Displacement, "Disp.");
                        });

                        // Show stress-component picker when in Stress mode
                        if self.overlay_mode == OverlayMode::Stress {
                            ui.label(egui::RichText::new("Stress Component").small());
                            egui::Grid::new("stress_comp_grid").spacing([8.0, 4.0]).show(ui, |ui| {
                                let mut count = 0;
                                for component in nalgebra_ext::matrix3::Component::iter() {
                                    ui.radio_value(
                                        &mut self.stress_tensor_component,
                                        component,
                                        egui::RichText::new(component.to_string()).small(),
                                    );
                                    count += 1;
                                    if count % 3 == 0 { ui.end_row(); }
                                }
                            });
                        }

                        // 4.5 Colormap
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("Colormap").small().strong());
                        egui::ComboBox::from_id_source("colormap_combo")
                            .selected_text(self.colormap.label())
                            .show_ui(ui, |ui| {
                                for cm in [Colormap::CoolWarm, Colormap::Viridis,
                                           Colormap::Plasma, Colormap::Jet, Colormap::Grayscale] {
                                    ui.selectable_value(&mut self.colormap, cm, cm.label());
                                }
                            });

                        ui.separator();

                        ui.checkbox(&mut self.show_stress_gradients, "Render Colour Overlay");
                        ui.checkbox(&mut self.show_broken_elements, "Highlight Fracture Zones");
                        ui.checkbox(&mut self.show_force_vectors, "Visualize Force Vectors");

                        ui.separator();

                        // 4.3 Slice plane — X/Y/Z axis + offset
                        ui.checkbox(&mut self.slice_enabled, "Enable Slice Plane");
                        if self.slice_enabled {
                            ui.indent("slice_indent", |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Axis:");
                                    ui.radio_value(&mut self.slice_axis, SliceAxis::X, "X");
                                    ui.radio_value(&mut self.slice_axis, SliceAxis::Y, "Y");
                                    ui.radio_value(&mut self.slice_axis, SliceAxis::Z, "Z");
                                });
                                ui.add(egui::Slider::new(&mut self.slice_offset, -10.0..=10.0)
                                    .text("Offset"));
                            });
                        }
                    });
                        
                    ui.add_space(12.0);
                    
                    // --- View & Navigation ---
                    super::premium::premium_card(ui, "🎮 Viewport", |ui| {
                        ui.vertical_centered_justified(|ui| {
                            if ui.button("⟳ Reset Camera View").clicked() {
                                self.rotation_x = -0.5;
                                self.rotation_y = 0.5;
                            }
                        });
                        ui.add_space(4.0);
                        ui.vertical(|ui| {
                            ui.small("🖱 Rotate: Right-click + Drag");
                            ui.small("🔍 Zoom: Ctrl + Scroll");
                            ui.small("✋ Pan: Left-click + Drag");
                        });
                    });

                    ui.add_space(20.0);
                });
            });
    }

    fn record_stress_data_of_element(&mut self, index: usize) {
        if self.engine_alive {
            self.engine.record_stress_data_of_element(index);
        }
        self.selected_element_index = Some(index);
    }

    fn record_stress_data(&mut self, index: usize) {
        puffin::profile_function!();
        if self.selected_element_index == Some(index) {
            return;
        }
        self.stop_recording_stress_data();
        self.record_stress_data_of_element(index);
    }

    fn stop_recording_stress_data(&mut self) {
        puffin::profile_function!();
        if let Some(handle) = self.selected_element_index.take() {
            if self.engine_alive {
                self.engine.stop_recording_stress_data(handle);
            }
        }
    }

    fn record_vertex_position(&mut self, index: usize) {
        if self.selected_vertex_index == Some(index) {
            return;
        }
        self.stop_recording_vertex_position();
        if self.engine_alive {
            self.engine.record_vertex_position(index);
        }
        self.selected_vertex_index = Some(index);
    }

    fn stop_recording_vertex_position(&mut self) {
        puffin::profile_function!();
        if let Some(index) = self.selected_vertex_index.take() {
            if self.engine_alive {
                self.engine.stop_recording_vertex_position(index);
            }
        }
    }

    fn add_controls(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        use crate::model::engine::State;
        let state = self.receivers.state_receiver.data;

        if state == State::Unconfigured {
            ui.horizontal(|ui| {
                super::premium::status_dot(ui, false);
                ui.label(egui::RichText::new("Configure physics to begin simulation").italics().weak());
            });
            return;
        }

        // Smooth transition: animate the control bar background tint on state change
        let is_running = state == State::Running;
        let run_anim = ui.ctx().animate_bool_with_time(
            egui::Id::new("sim_run_anim"), is_running, 0.4
        );
        // Paint a subtle animated accent glow behind the control bar
        let bar_rect = ui.max_rect();
        let glow_alpha = (run_anim * 18.0) as u8;
        ui.painter().rect_filled(
            bar_rect,
            0.0,
            egui::Color32::from_rgba_unmultiplied(99, 102, 241, glow_alpha),
        );

        if !self.engine_alive {
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::from_rgb(248, 113, 113), "⚠️ Simulation engine disconnected. The physics worker may have encountered an error.");
                if ui.button("Details").clicked() {
                    // Show last error if any
                }
            });
            return;
        }

        ui.horizontal(|ui| {
            // --- Left: Status dot + Simulation Stats ---
            ui.horizontal(|ui| {
                super::premium::status_dot(ui, is_running);
                ui.add_space(4.0);

                let state_label = match state {
                    State::Running => egui::RichText::new("RUNNING").size(10.5)
                        .color(egui::Color32::from_rgb(74, 222, 128)).strong(),
                    State::Paused  => egui::RichText::new("PAUSED").size(10.5)
                        .color(egui::Color32::from_rgb(251, 191, 36)).strong(),
                    _              => egui::RichText::new("READY").size(10.5).weak(),
                };
                ui.label(state_label);

                ui.add_space(16.0);

                if let Some(frame) = self.receivers.frame_receiver.data.as_ref() {
                    let data = frame.data();
                    ui.label(egui::RichText::new(format!("{} {}", unicode_symbols::BULLET, data.nodes().len())).small().weak());
                    ui.label(egui::RichText::new("nodes").small().weak());
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(format!("🔺 {}", data.elements().len())).small().weak());
                    ui.label(egui::RichText::new("elements").small().weak());
                    let broken = data.elements().iter().filter(|e| *e.is_broken()).count();
                    if broken > 0 {
                        ui.add_space(8.0);
                        ui.colored_label(egui::Color32::from_rgb(248, 113, 113), format!("💔 {broken} broken"));
                    }
                    let inverted = data.elements().iter().filter(|e| *e.is_inverted()).count();
                    if inverted > 0 {
                        ui.add_space(8.0);
                        ui.colored_label(egui::Color32::from_rgb(251, 191, 36), format!("⚠️ {inverted} inverted"));
                    }
                }
            });

            // --- Center: Playback HUD ---
            let available_width = ui.available_width();
            let controls_width = 120.0;
            ui.add_space((available_width - controls_width) / 2.0);
            
            ui.horizontal(|ui| {
                let frame = self.receivers.frame_receiver.data.as_ref();
                let progress = frame.map(|f| *f.progress()).unwrap_or_default();
                let progress_f = if progress.is_finite() { progress } else { 0.0 };
                
                if progress_f > 0.0 {
                    if ui.button(egui::RichText::new(unicode_symbols::REFRESH).size(16.0))
                        .on_hover_text("Reset simulation").clicked() {
                        self.engine.rewind();
                    }
                }
                self.playback_toggle(ui, state);
            });
            
            ui.add_space((available_width - controls_width) / 2.0);

            // --- Right: Animated Progress + Runtime ---
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let frame = self.receivers.frame_receiver.data.as_ref();

                if let Some(runtime) = frame.and_then(|f| *f.runtime()) {
                    ui.label(egui::RichText::new(format!("{runtime:#?}")).small().weak());
                    ui.label(egui::RichText::new("runtime:").small().weak());
                }
                ui.add_space(12.0);

                if let Some(f) = frame {
                    if let Some(total) = f.total_iterations() {
                        let progress = *f.progress();

                        // Smooth color lerp: blue when idle → indigo-violet when running
                        let r = egui::lerp(79.0..=99.0, run_anim) as u8;
                        let g = egui::lerp(70.0..=60.0, run_anim) as u8;
                        let b = egui::lerp(229.0..=241.0, run_anim) as u8;
                        let bar_color = egui::Color32::from_rgb(r, g, b);

                        ui.add(egui::ProgressBar::new(progress)
                            .show_percentage()
                            .desired_width(120.0)
                            .rounding(egui::Rounding::same(4.0))
                            .fill(bar_color)
                        );
                        ui.label(egui::RichText::new(format!("{}/{}", f.iterations(), total)).small().weak());
                    }
                }
            });
        });
    }

    fn playback_toggle(&mut self, ui: &mut Ui, state: State) {
        puffin::profile_function!();
        let symbol = if state == State::Running {
            unicode_symbols::PAUSE
        } else if state == State::Paused {
            unicode_symbols::PLAY
        } else {
            return;
        };
        shortcut!(PLAYBACK, Modifiers::NONE, Key::Space);
        
        let button = Button::new(egui::RichText::new(symbol).size(20.0))
            .shortcut_text(ui.ctx().format_shortcut(&PLAYBACK_SHORTCUT))
            .rounding(egui::Rounding::same(6.0))
            .stroke(ui.visuals().widgets.inactive.bg_stroke);
            
        let response = ui.add(button)
            .on_hover_text(if state == State::Running {
                "Pause simulation (Space)"
            } else {
                "Resume simulation (Space)"
            });
            
        let toggle_playback = response.clicked() || super::consume_shortcut(ui, &PLAYBACK_SHORTCUT);
        if !toggle_playback {
            return;
        }
        if state == State::Running {
            self.engine.pause();
        } else {
            self.engine.play();
        }
    }

    fn plot_series_for_vector(
        series: &[TimeStampedValue<Vector3<f32>>],
        index: usize,
    ) -> Vec<[f64; 2]> {
        puffin::profile_function!();
        series
            .par_iter()
            .map(|value| [*value.time_stamp() as f64, value.value()[index] as f64])
            .filter(|p| p[0].is_finite() && p[1].is_finite())
            .collect()
    }

    fn default_open_collapsing_plot<T, I, S>(
        heading: T,
        ui: &mut Ui,
        plot_id: I,
        plot_cursor_group_id: &'static str,
        plot_y_label: &'static str,
        plot_series: S,
        plot_color: Color32,
    ) where
        T: Into<WidgetText>,
        I: Hash,
        S: Fn() -> Vec<[f64; 2]>,
    {
        CollapsingHeader::new(heading)
            .default_open(true)
            .show(ui, |ui| {
                Plot::new(plot_id)
                    .link_cursor(plot_cursor_group_id, true, true)
                    .show_grid(false)
                    .custom_x_axes(vec![AxisHints::new_x().label("Duration")])
                    .custom_y_axes(vec![AxisHints::new_y().label(plot_y_label)])
                    .show(ui, |ui| {
                        ui.line(Line::new(plot_series()).color(plot_color));
                    })
            });
    }

    fn show_time_series_displacement_plot(
        &self,
        ui: &mut Ui,
        series: &[TimeStampedValue<Vector3<f32>>],
    ) {
        puffin::profile_function!();
        ui.label("Displacement plot");
        macro_rules! id_source {
            ( $comp:literal ) => {
                const_format::formatcp!("simulation_displacement_{}_plot", $comp)
            };
        }
        macro_rules! plot {
            ( $desired_size:expr, $comp:literal, $index:expr, $color:expr ) => {
                ui.allocate_ui($desired_size, |ui| {
                    Self::default_open_collapsing_plot(
                        $comp,
                        ui,
                        id_source!($comp),
                        "simulation_displacement_plot_group",
                        "Displacement",
                        || Self::plot_series_for_vector(series, $index),
                        $color,
                    )
                });
            };
        }
        let size = ui.available_size();
        let desired_size = Vec2::new(size.x, size.y / 3.0 - ui.spacing().item_spacing.y);
        plot!(desired_size, "Dx", 0, Color32::RED);
        plot!(desired_size, "Dy", 1, Color32::YELLOW);
        plot!(desired_size, "Dz", 2, Color32::GREEN);
    }

    fn plot_series_for_stress_component(
        series: &[TimeStampedValue<Matrix3<f32>>],
        component: Component,
    ) -> Vec<[f64; 2]> {
        puffin::profile_function!();
        series
            .par_iter()
            .map(|value| {
                [
                    *value.time_stamp() as f64,
                    *value.value().index(component) as f64,
                ]
            })
            .filter(|p| p[0].is_finite() && p[1].is_finite())
            .collect()
    }

    fn show_time_series_stress_plot(&self, ui: &mut Ui, series: &[TimeStampedValue<Matrix3<f32>>]) {
        puffin::profile_function!();
        ui.label("Stress plot");
        let size = ui.available_size();
        let desired_size = Vec2::new(size.x, size.y / 4.0 - ui.spacing().item_spacing.y);
        egui::ScrollArea::vertical().show(ui, |ui| {
            Component::iter().for_each(|component| {
                ui.allocate_ui(desired_size, |ui| {
                    Self::default_open_collapsing_plot(
                        format!("E{component}"),
                        ui,
                        format!("simulation_stress_{component}_plot"),
                        "simulation_stress_plot_group",
                        "Stress",
                        || Self::plot_series_for_stress_component(series, component),
                        Self::color_for_component(component),
                    )
                });
            });
        });
    }

    fn show_time_series_boundary_average_data_plot(&self, ui: &mut Ui, data: &BoundaryAverage) {
        puffin::profile_function!();
        ui.label("Boundary average plot");
        let size = ui.available_size();
        macro_rules! id_source {
            ( $comp:literal ) => {
                const_format::formatcp!("simulation_boundary_average_{}_plot", $comp)
            };
        }
        macro_rules! vector_series {
            ( $series:expr, $index:expr ) => {
                || {
                    $series
                        .par_iter()
                        .map(|tsv| [*tsv.time_stamp() as f64, tsv.value()[$index] as f64])
                        .filter(|p| p[0].is_finite() && p[1].is_finite())
                        .collect()
                }
            };
            ($series:expr, $kind:ident, $index:expr) => {
                || {
                    $series
                        .par_iter()
                        .map(|tsv| [*tsv.time_stamp() as f64, tsv.value().$kind()[$index] as f64])
                        .filter(|p| p[0].is_finite() && p[1].is_finite())
                        .collect()
                }
            };
        }
        macro_rules! vector_plot {
            ($desired_size:expr, $vector_name:literal, $comp:literal, $series:expr, $color:expr) => {
                ui.allocate_ui($desired_size, |ui| {
                    Self::default_open_collapsing_plot(
                        $comp,
                        ui,
                        id_source!($comp),
                        "simulation_boundary_average_plot_group",
                        $vector_name,
                        $series,
                        $color,
                    );
                });
            };
        }
        match data {
            BoundaryAverage::Force(series) => {
                let desired_size = Vec2::new(size.x, size.y / 3.0 - ui.spacing().item_spacing.y);
                vector_plot!(
                    desired_size,
                    "Force",
                    "Fx",
                    vector_series!(series, 0),
                    Color32::RED
                );
                vector_plot!(
                    desired_size,
                    "Force",
                    "Fy",
                    vector_series!(series, 1),
                    Color32::YELLOW
                );
                vector_plot!(
                    desired_size,
                    "Force",
                    "Fz",
                    vector_series!(series, 2),
                    Color32::GREEN
                );
            }
            BoundaryAverage::Displacement(series) => {
                let desired_size = Vec2::new(size.x, size.y / 3.0 - ui.spacing().item_spacing.y);
                vector_plot!(
                    desired_size,
                    "Displacement",
                    "Dx",
                    vector_series!(series, 0),
                    Color32::RED
                );
                vector_plot!(
                    desired_size,
                    "Displacement",
                    "Dy",
                    vector_series!(series, 1),
                    Color32::YELLOW
                );
                vector_plot!(
                    desired_size,
                    "Displacement",
                    "Dz",
                    vector_series!(series, 2),
                    Color32::GREEN
                );
            }
            BoundaryAverage::ForceAndDisplacement(series) => {
                let desired_size = Vec2::new(size.x, size.y / 6.0 - ui.spacing().item_spacing.y);
                vector_plot!(
                    desired_size,
                    "Force",
                    "Fx",
                    vector_series!(series, force, 0),
                    Color32::RED
                );
                vector_plot!(
                    desired_size,
                    "Force",
                    "Fy",
                    vector_series!(series, force, 1),
                    ORANGE
                );
                vector_plot!(
                    desired_size,
                    "Force",
                    "Fz",
                    vector_series!(series, force, 2),
                    Color32::KHAKI
                );
                vector_plot!(
                    desired_size,
                    "Displacement",
                    "Dx",
                    vector_series!(series, displacement, 0),
                    Color32::YELLOW
                );
                vector_plot!(
                    desired_size,
                    "Displacement",
                    "Dy",
                    vector_series!(series, displacement, 1),
                    Color32::LIGHT_GREEN
                );
                vector_plot!(
                    desired_size,
                    "Displacement",
                    "Dz",
                    vector_series!(series, displacement, 2),
                    Color32::GREEN
                );
            }
        }
    }

    fn show_preview(&mut self, ui: &mut Ui) -> FramePreviewResponse {
        puffin::profile_function!();
        CentralPanel::default()
            .frame(egui::Frame::default())
            .show_inside(ui, |ui| self.preview_contents(ui))
            .inner
    }




    fn preview_contents(&mut self, ui: &mut Ui) -> FramePreviewResponse {
        puffin::profile_function!();
        let auto_bounds = self.receivers.frame_receiver.data.is_none();
        let plot_config = || {
            plot_utils::plot_without_clutter("simulation_preview_plot")
                .data_aspect(1.0)
                .auto_bounds(Vec2b::new(auto_bounds, auto_bounds))
                .show_axes(false)
                .allow_double_click_reset(false)
        };

        let frame = self.receivers.frame_receiver.data.clone();
        let Some(frame) = frame else {
            plot_config().show(ui, |ui| {
                let geometry = self.engine.polygon_data().plot_geometry();
                plot_utils::plot_cached_geometry(ui, geometry, plot_utils::default_transform);
            });
            return FramePreviewResponse::Noop;
        };

        let data = frame.data();
        let projector = Projector::new(self.rotation_x, self.rotation_y);
        
        // Optimization: Parallel projection of all nodes once per frame
        let projected_nodes: Vec<[f64; 2]> = data.nodes().par_iter().map(|node| {
            let p = node.position();
            projector.project([p.x, p.y, p.z])
        }).collect();

        let plot_response = plot_config().show(ui, |ui| {
            if ui.response().dragged_by(egui::PointerButton::Secondary) {
                let delta = ui.response().drag_delta();
                self.rotation_y += delta.x * 0.01;
                self.rotation_x += delta.y * 0.01;
            }

            if self.auto_rotate {
                self.rotation_y += 0.01;
                ui.ctx().request_repaint();
            }
            
            // Ensure camera rotations are always finite
            if !self.rotation_x.is_finite() { self.rotation_x = -0.5; }
            if !self.rotation_y.is_finite() { self.rotation_y = 0.5; }

            let result = self.plot_frame_optimized(ui, data, &projected_nodes, &projector);
            gnomon::draw_gnomon(ui, &mut self.rotation_x, &mut self.rotation_y);
            result
        });
        
        let response = plot_response.response;
        let response = match plot_response.inner {
            FramePlotHoverResponse::Noop => response,
            FramePlotHoverResponse::ElementIndex(index) => response.on_hover_ui_at_pointer(|ui| {
                let element = &frame.data().elements()[index];
                
                ui.label(egui::RichText::new(format!("Element {}", index)).strong());
                ui.separator();
                
                ui.label(Self::format_stress(
                    *element.stress().index(self.stress_tensor_component),
                ));
                
                // Add Von Mises stress to tooltip
                let s = element.stress();
                let sxx = s[(0,0)]; let syy = s[(1,1)]; let szz = s[(2,2)];
                let sxy = s[(0,1)]; let syz = s[(1,2)]; let sxz = s[(0,2)];
                let vm = (0.5 * ((sxx-syy).powi(2) + (syy-szz).powi(2) + (szz-sxx).powi(2))
                  + 3.0*(sxy*sxy + syz*syz + sxz*sxz)).sqrt();
                ui.label(egui::RichText::new(format!("Von Mises: {:.2e} Pa", vm)).weak());

                ui.label(format!("Strain energy: {:.3e} J", element.strain_energy()));
                
                if *element.is_broken() {
                    ui.label(egui::RichText::new("⚠️ Fractured").color(Color32::RED));
                } else {
                    ui.label(egui::RichText::new("✅ Intact").color(Color32::GREEN));
                }
                
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Click to plot stress series").small().italics());
            }),
            FramePlotHoverResponse::VertexIndex(index) => response.on_hover_ui_at_pointer(|ui| {
                let node = &frame.data().nodes()[index];
                
                ui.label(egui::RichText::new(format!("Node {}", index)).strong());
                ui.separator();
                
                let p = node.position();
                let d = p - node.initial_position();
                ui.label(format!("Position: {:.2}i + {:.2}j + {:.2}k", p.x, p.y, p.z));
                ui.label(format!("Displacement: {:.3e} m", d.norm()));
                ui.label(format!("Velocity: {:.3e} m/s", node.velocity().norm()));
                ui.label(format!("Net Force: {:.3e} N", node.force().norm()));
                
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Click to plot displacement series").small().italics());
            }),
        };

        if !response.clicked() {
            return FramePreviewResponse::Noop;
        };

        match plot_response.inner {
            FramePlotHoverResponse::Noop => FramePreviewResponse::Noop,
            FramePlotHoverResponse::ElementIndex(handle) => {
                FramePreviewResponse::ElementSelected(handle)
            }
            FramePlotHoverResponse::VertexIndex(index) => {
                FramePreviewResponse::VertexSelected(index)
            }
        }
    }


    fn p_is_inside_abc(a: &[f32; 2], b: &[f32; 2], c: &[f32; 2], p: [f32; 2]) -> bool {
        puffin::profile_function!();
        let d = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1]);
        let ba = ((b[1] - c[1]) * (p[0] - c[0]) + (c[0] - b[0]) * (p[1] - c[1])) / d;
        let bb = ((c[1] - a[1]) * (p[0] - c[0]) + (a[0] - c[0]) * (p[1] - c[1])) / d;
        let bc = 1.0 - ba - bb;
        ba >= 0.0 && bb >= 0.0 && bc >= 0.0
    }

    fn format_stress(stress: f32) -> String {
        puffin::profile_function!();
        let stress_abs = stress.abs();
        let sign = if stress_abs == 0.0 || stress.is_sign_positive() {
            char::default()
        } else {
            '-'
        };
        let stress = stress_abs;
        macro_rules! fmt {
            ( $stress:expr, $unit:expr ) => {
                format!("Stress: {sign}{:.2} {}", $stress, $unit)
            };
        }
        if stress >= 9e8 {
            fmt!(stress / 1e9, "GPa")
        } else if stress >= 9e5 {
            fmt!(stress / 1e6, "MPa")
        } else if stress >= 9e2 {
            fmt!(stress / 1e3, "kPa")
        } else {
            fmt!(stress, "Pa")
        }
    }


    fn plot_frame_optimized(
        &self, 
        ui: &mut PlotUi, 
        data: &ExportData, 
        projected_nodes: &[[f64; 2]],
        projector: &Projector
    ) -> FramePlotHoverResponse {
        puffin::profile_function!();
        
        let result = self.plot_mesh_optimized(ui, data, projected_nodes);
        
        if self.show_force_vectors {
            self.plot_force_vectors_optimized(ui, data, projected_nodes, projector);
        }
        if self.show_stress_gradients {
            self.plot_colorbar(ui, data);
        }
        result
    }

    fn plot_mesh_optimized(
        &self, 
        ui: &mut PlotUi, 
        data: &ExportData,
        projected_nodes: &[[f64; 2]]
    ) -> FramePlotHoverResponse {
        puffin::profile_function!();
        let stress_comp = self.stress_tensor_component;
        let min_stress = *data.min_stress().index(stress_comp);
        let max_stress = *data.max_stress().index(stress_comp);
        let stress_range = (max_stress - min_stress).max(1e-10);

        // Max values for normalisation
        let max_disp: f32 = if self.overlay_mode == OverlayMode::Displacement {
            data.nodes().iter()
                .map(|n| (n.position() - n.initial_position()).norm())
                .fold(0.0f32, f32::max)
                .max(1e-10)
        } else { 1.0 };

        let von_mises_of = |s: &Matrix3<f32>| -> f32 {
            let sxx = s[(0,0)]; let syy = s[(1,1)]; let szz = s[(2,2)];
            let sxy = s[(0,1)]; let syz = s[(1,2)]; let sxz = s[(0,2)];
            (0.5 * ((sxx-syy).powi(2) + (syy-szz).powi(2) + (szz-sxx).powi(2))
              + 3.0*(sxy*sxy + syz*syz + sxz*sxz)).sqrt()
        };
        let max_vm: f32 = if self.overlay_mode == OverlayMode::VonMises {
            data.elements().iter()
                .map(|e| von_mises_of(e.stress()))
                .fold(0.0f32, f32::max)
                .max(1e-10)
        } else { 1.0 };
        
        let mut hover_result = FramePlotHoverResponse::Noop;
        let pointer_coords = ui.pointer_coordinate();

        // 1. Adaptive Clipping: Calculate a limit to hide exploding nodes while showing the model.
        let mut radii: Vec<f64> = projected_nodes.iter()
            .map(|p| (p[0]*p[0] + p[1]*p[1]).sqrt())
            .filter(|r| r.is_finite())
            .collect();
        
        let clip_limit = if !radii.is_empty() {
            radii.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let median_radius = radii[radii.len() / 2];
            (median_radius * 3.5).max(10.0) // Tightened clipping to 3.5x to prevent numerical instability
        } else {
            1000.0
        };

        // 2. Prepare and sort elements by depth
        let mut sorted_elements: Vec<_> = data.elements().iter().enumerate()
            .filter(|(_, element)| {
                if *element.is_broken() && !self.show_broken_elements {
                    return false;
                }
                
                let indices = element.indices();
                for &i in indices {
                    let p = projected_nodes[i];
                    if !p[0].is_finite() || !p[1].is_finite() || p[0].abs() > clip_limit || p[1].abs() > clip_limit {
                        return false;
                    }
                }

                if self.slice_enabled {
                    let p0 = data.node_position(indices[0]);
                    let p1 = data.node_position(indices[1]);
                    let p2 = data.node_position(indices[2]);
                    let p3 = data.node_position(indices[3]);
                    let coord = match self.slice_axis {
                        SliceAxis::X => (p0[0] + p1[0] + p2[0] + p3[0]) / 4.0,
                        SliceAxis::Y => (p0[1] + p1[1] + p2[1] + p3[1]) / 4.0,
                        SliceAxis::Z => (p0[2] + p1[2] + p2[2] + p3[2]) / 4.0,
                    };
                    if coord < self.slice_offset { return false; }
                }
                true
            })
            .map(|(index, element)| {
                let indices = element.indices();
                let z_avg = (data.node_position(indices[0])[2] + 
                             data.node_position(indices[1])[2] + 
                             data.node_position(indices[2])[2] + 
                             data.node_position(indices[3])[2]) / 4.0;
                
                let (_sy, cy) = self.rotation_y.sin_cos();
                let view_z = z_avg * cy;
                (index, element, view_z)
            })
            .collect();

        sorted_elements.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

        // 3. Draw elements
        let on_primary = super::on_primary_color(ui.ctx());
        // Matte premium finish for solid mode (less transparent, more contrast)
        let default_fill = egui::Color32::from_gray(160).linear_multiply(0.85); 
        let default_stroke = egui::Stroke::new(0.3, on_primary.linear_multiply(0.4));

        for (index, element, view_z) in sorted_elements {
            let is_broken = *element.is_broken();
            let indices = element.indices();

            let p0 = projected_nodes[indices[0]];
            let p1 = projected_nodes[indices[1]];
            let p2 = projected_nodes[indices[2]];
            let p3 = projected_nodes[indices[3]];

            let (color, stroke) = if !self.show_stress_gradients {
                (default_fill, default_stroke)
            } else if is_broken {
                let t = ui.ctx().input(|i| i.time) as f32;
                let pulse = (t * 3.0).sin() * 0.5 + 0.5;
                let alpha = (120.0 + pulse * 80.0) as u8;
                (egui::Color32::from_rgba_unmultiplied(248, 113, 113, alpha), egui::Stroke::new(0.5, Color32::DARK_RED))
            } else {
                let t = match self.overlay_mode {
                    OverlayMode::Stress => {
                        ((*element.stress().index(stress_comp)) - min_stress) / stress_range
                    }
                    OverlayMode::VonMises => {
                        von_mises_of(element.stress()) / max_vm
                    }
                    OverlayMode::Displacement => {
                        let ni = element.indices();
                        let d: f32 = ni.iter()
                            .map(|&i| (data.nodes()[i].position() - data.nodes()[i].initial_position()).norm())
                            .sum::<f32>() / 4.0;
                        d / max_disp
                    }
                };
                let t_f = if t.is_finite() { t } else { 0.0 };
                let base = self.colormap.map(t_f.clamp(0.0, 1.0));
                let view_z_f = if view_z.is_finite() { view_z as f32 } else { 0.0 };
                let depth_factor = (view_z_f * 0.5 + 0.5).clamp(0.55, 1.0);
                let [r, g, b, a] = base.to_array();
                let fill = egui::Color32::from_rgba_unmultiplied(
                    (r as f32 * depth_factor) as u8,
                    (g as f32 * depth_factor) as u8,
                    (b as f32 * depth_factor) as u8,
                    a,
                );
                (fill, egui::Stroke::NONE)
            };

            let faces = [[p0, p1, p2], [p0, p2, p3], [p0, p3, p1], [p1, p3, p2]];
            
            for face in &faces {
                // Backface culling
                let edge1_x = face[1][0] - face[0][0];
                let edge1_y = face[1][1] - face[0][1];
                let edge2_x = face[2][0] - face[0][0];
                let edge2_y = face[2][1] - face[0][1];
                let normal_z = edge1_x * edge2_y - edge1_y * edge2_x;

                // Guard: skip degenerate (zero-area / collinear) triangles.
                // egui's internal hit_test unwraps a bounding-box Option that is
                // None for such polygons, causing an unconditional panic at
                // egui/src/hit_test.rs:265. A threshold of 1e-10 is safe.
                if normal_z > 1e-10 {
                    ui.polygon(egui_plot::Polygon::new(face.to_vec())
                        .fill_color(color)
                        .stroke(stroke));
                }
            }
            
            // Hover detection with numerical safety
            if let Some(coords) = pointer_coords {
                let p = [coords.x as f32, coords.y as f32];
                if p[0].is_finite() && p[1].is_finite() {
                    let pf0 = [p0[0] as f32, p0[1] as f32];
                    let pf1 = [p1[0] as f32, p1[1] as f32];
                    let pf2 = [p2[0] as f32, p2[1] as f32];
                    let pf3 = [p3[0] as f32, p3[1] as f32];

                    if Self::p_is_inside_abc(&pf0, &pf1, &pf2, p) ||
                       Self::p_is_inside_abc(&pf0, &pf1, &pf3, p) ||
                       Self::p_is_inside_abc(&pf0, &pf2, &pf3, p) ||
                       Self::p_is_inside_abc(&pf1, &pf2, &pf3, p) {
                        hover_result = FramePlotHoverResponse::ElementIndex(index);
                    }
                }
            }
        }
        
        hover_result
    }

    fn plot_force_vectors_optimized(
        &self, 
        ui: &mut PlotUi, 
        data: &ExportData,
        projected_nodes: &[[f64; 2]],
        projector: &Projector
    ) {
        puffin::profile_function!();
        let max_force: f32 = data.nodes()
            .iter()
            .map(|n| n.force().norm())
            .fold(0.0_f32, f32::max);
        if max_force < 1e-6 { return; }
        let scale = 0.3 / max_force;

        for (i, node) in data.nodes().iter().enumerate() {
            let f = node.force();
            if f.norm_squared() < 1e-8 { continue; }

            let p = node.position();
            let tip = [p.x + f.x * scale, p.y + f.y * scale, p.z + f.z * scale];
            let from = projected_nodes[i];
            let to = projector.project(tip);
            
            // Guard against non-finite or degenerate lines
            if from[0].is_finite() && from[1].is_finite() && to[0].is_finite() && to[1].is_finite() {
                if (to[0] - from[0]).abs() > 1e-10 || (to[1] - from[1]).abs() > 1e-10 {
                    ui.line(Line::new(vec![from, to]).color(ORANGE).width(0.8));
                }
            }
        }
    }


    /// Draw a stress colorbar in the top-right corner of the plot
    fn plot_colorbar(&self, ui: &mut PlotUi, data: &ExportData) {
        puffin::profile_function!();
        
        let bounds = ui.plot_bounds();
        if !bounds.min()[0].is_finite() || !bounds.max()[0].is_finite() || 
           !bounds.min()[1].is_finite() || !bounds.max()[1].is_finite() {
            return;
        }

        let (min_val, max_val, title) = match self.overlay_mode {
            OverlayMode::Stress => {
                let min_stress = *data.min_stress().index(self.stress_tensor_component);
                let max_stress = *data.max_stress().index(self.stress_tensor_component);
                (min_stress, max_stress, format!("Stress ({})", self.stress_tensor_component))
            }
            OverlayMode::VonMises => {
                let von_mises_of = |s: &Matrix3<f32>| -> f32 {
                    let sxx = s[(0,0)]; let syy = s[(1,1)]; let szz = s[(2,2)];
                    let sxy = s[(0,1)]; let syz = s[(1,2)]; let sxz = s[(0,2)];
                    (0.5 * ((sxx-syy).powi(2) + (syy-szz).powi(2) + (szz-sxx).powi(2))
                      + 3.0*(sxy*sxy + syz*syz + sxz*sxz)).sqrt()
                };
                let max_vm: f32 = data.elements().iter()
                    .map(|e| von_mises_of(e.stress()))
                    .fold(0.0f32, f32::max)
                    .max(1e-10);
                (0.0, max_vm, "von Mises (Pa)".to_string())
            }
            OverlayMode::Displacement => {
                let max_disp: f32 = data.nodes().iter()
                    .map(|n| (n.position() - n.initial_position()).norm())
                    .fold(0.0f32, f32::max)
                    .max(1e-10);
                (0.0, max_disp, "Disp. (m)".to_string())
            }
        };

        // Draw 20 stacked colored rectangles to form a gradient bar
        let steps = 20usize;
        let bounds = ui.plot_bounds();
        let bar_x_min = bounds.max()[0] - (bounds.max()[0] - bounds.min()[0]) * 0.06;
        let bar_x_max = bounds.max()[0] - (bounds.max()[0] - bounds.min()[0]) * 0.02;
        let bar_y_min = bounds.min()[1] + (bounds.max()[1] - bounds.min()[1]) * 0.05;
        let bar_y_max = bounds.max()[1] - (bounds.max()[1] - bounds.min()[1]) * 0.05;
        let step_h = (bar_y_max - bar_y_min) / steps as f64;

        // Guard: skip colorbar if the plot bounds are degenerate (zero area).
        // A zero step_h would produce rectangles with identical top/bottom
        // points, triggering the same egui hit_test.rs:265 unwrap() panic.
        if step_h.abs() < 1e-10 || !step_h.is_finite() || 
           (bar_x_max - bar_x_min).abs() < 1e-10 || !bar_x_max.is_finite() {
            return;
        }

        for i in 0..steps {
            let t = i as f32 / (steps - 1) as f32;
            let color = self.colormap.map(t);
            let y_bot = bar_y_min + step_h * i as f64;
            let y_top = y_bot + step_h;
            ui.polygon(
                Polygon::new(vec![
                    [bar_x_min, y_bot], [bar_x_max, y_bot],
                    [bar_x_max, y_top], [bar_x_min, y_top],
                ])
                .fill_color(color)
                .stroke(Stroke::NONE),
            );
        }

        // Labels at top (max) and bottom (min)
        let fmt = |v: f32| {
            if self.overlay_mode == OverlayMode::Displacement {
                format!("{:.3e}", v)
            } else {
                Self::format_stress(v).replace("Stress: ", "")
            }
        };
        ui.text(egui_plot::Text::new(
            PlotPoint::new(bar_x_min, bar_y_max + step_h * 1.5),
            egui::RichText::new(title).size(10.0).strong(),
        ));
        ui.text(egui_plot::Text::new(
            PlotPoint::new(bar_x_min, bar_y_max),
            egui::RichText::new(fmt(max_val)).size(9.0).strong(),
        ));
        ui.text(egui_plot::Text::new(
            PlotPoint::new(bar_x_min, bar_y_min),
            egui::RichText::new(fmt(min_val)).size(9.0).strong(),
        ));
    }

    #[must_use]
    fn input_dialog_and_error_ui(
        ui: &mut Ui,
        dialog_state: &mut Option<config_dialog::State>,
        configure_error: &mut Option<String>,
        gpu_available: bool,
    ) -> Option<Box<Config>> {
        if let Some(err) = configure_error {
            if error_dialog::show(err, ui.ctx()).closed() {
                *configure_error = None;
            }
            return None;
        }

        let mut state = dialog_state.take()?;
        state.set_gpu_available(gpu_available);
        use config_dialog::Response;
        match config_dialog::show(&mut state, ui.ctx()) {
            Response::Noop => {
                *dialog_state = Some(state);
                None
            }
            Response::ConfigResult(result) => match result {
                Ok(config) => Some(config),
                Err(err) => {
                    *dialog_state = Some(state);
                    *configure_error = err.into();
                    None
                }
            },
            Response::Cancel => None,
        }
    }
}

mod serde_impl {
    use super::*;
    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Debug, Serialize, Deserialize)]
    enum WrappedProjectData {
        WithCpdData(Box<Data<WithCpdExportData>>),
        WithMesh(Data<WithMesh>),
    }

    impl Serialize for Page {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let project_data = self.engine.project_data().clone();
            let project_data = match &self.receivers.frame_receiver.data {
                Some(frame) => WrappedProjectData::WithCpdData(Box::new(
                    project_data.with_export_data(frame.data().clone()),
                )),
                None => WrappedProjectData::WithMesh(project_data),
            };
            project_data.serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Page {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            WrappedProjectData::deserialize(deserializer).and_then(|wpd| {
                match wpd {
                    WrappedProjectData::WithCpdData(project_data) => Page::try_from(*project_data),
                    WrappedProjectData::WithMesh(project_data) => Ok(Page::from(project_data)),
                }
                .map_err(de::Error::custom)
            })
        }
    }
}
