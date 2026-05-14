mod dialog;

use super::{bottom_panel, error_dialog, plot_utils};
use crate::model::{
    boundary_conditions::{Axis3D, Configurator, FacePlaneCondition, PlaneComparison},
    project::data::{Data, WithBoundaryConditions, WithShape},
};
use cgal::{
    curve::{Curve, LineSegment},
    num::Algebraic,
    BoundaryId, Coordinate, Point, PolygonSet, PolygonWithHoles,
};
use cpd::boundary_condition::BoundaryCondition;
use ecolor::Color32;
use egui::{CentralPanel, Context, Frame, RichText, ScrollArea, SidePanel, Slider, Ui};
use egui_plot::{Line, MarkerShape, PlotPoint, PlotUi, Points, Text};
use std::{fmt::Debug, ops::RangeInclusive};
use strum::IntoEnumIterator;

const VIOLET: Color32 = Color32::from_rgb(0x8F, 0x00, 0xFF);

#[derive(Debug)]
struct SplitData {
    value: f64,
    range: RangeInclusive<f64>,
}

#[derive(Debug)]
enum SplitState {
    X(SplitData),
    Y(SplitData),
}

impl SplitState {
    fn value(&self) -> f64 {
        match self {
            SplitState::X(data) => data.value,
            SplitState::Y(data) => data.value,
        }
    }

    fn value_mut(&mut self) -> &mut f64 {
        match self {
            SplitState::X(data) => &mut data.value,
            SplitState::Y(data) => &mut data.value,
        }
    }

    fn range(&self) -> RangeInclusive<f64> {
        match self {
            SplitState::X(data) => data.range.clone(),
            SplitState::Y(data) => data.range.clone(),
        }
    }
}

impl From<&Curve> for SplitState {
    fn from(curve: &Curve) -> Self {
        macro_rules! split_data {
            ( $t:ident ) => {{
                let t1 = curve.end_points().start().$t().double_value();
                let t2 = curve.end_points().end().$t().double_value();
                let min = t1.min(t2);
                SplitData {
                    value: min,
                    range: min..=t1.max(t2),
                }
            }};
        }
        if matches!(curve, Curve::Line(LineSegment::Vertical(_))) {
            Self::Y(split_data!(y))
        } else {
            Self::X(split_data!(x))
        }
    }
}

#[derive(Debug)]
struct BoundaryState {
    id: BoundaryId,
    split_state: SplitState,
    point_fetch_error: Option<String>,
    show_point: bool,
}

impl BoundaryState {
    fn new(id: BoundaryId, configurator: &Configurator) -> Self {
        let curve = configurator
            .polygon_data()
            .polygon_set()
            .polygon_with_holes()[0]
            .boundary_with_id(&id);
        Self {
            id,
            split_state: SplitState::from(curve),
            point_fetch_error: None,
            show_point: false,
        }
    }
}

#[derive(Debug)]
pub struct Page {
    configurator: Box<Configurator>,
    boundary_state: Box<BoundaryState>,
    dialog_state: Option<Box<dialog::State>>,
    input_error: Option<String>,
    pending_face: FacePlaneCondition,
    rotation_x: f32,
    rotation_y: f32,
    needs_reset: bool,
}

#[derive(Debug)]
pub enum MenuResponse {
    Noop(Page),
    EditShape(Data<WithShape>),
}

#[derive(Debug)]
pub enum Response {
    Noop(Page),
    GenerateMesh(Data<WithBoundaryConditions>),
}

impl<T> From<T> for Page
where
    Configurator: From<T>,
{
    fn from(value: T) -> Self {
        let configurator = Configurator::from(value);
        Self {
            boundary_state: Box::new(BoundaryState::new(
                configurator.first_boundary_id(),
                &configurator,
            )),
            configurator: Box::new(configurator),
            dialog_state: None,
            input_error: None,
            pending_face: FacePlaneCondition::default(),
            rotation_x: -0.5,
            rotation_y: 0.5,
            needs_reset: false,
        }
    }
}

impl Page {
    #[must_use]
    pub fn add_menu_items(self, ui: &mut Ui) -> MenuResponse {
        puffin::profile_function!();
        let edit_shape = ui
            .menu_button("Edit", |ui| {
                if ui.button("Edit shape").clicked() {
                    ui.close_menu();
                    true
                } else {
                    false
                }
            })
            .inner
            .is_some_and(|clicked| clicked);
        if edit_shape {
            MenuResponse::EditShape(self.configurator.project_data_with_shape())
        } else {
            MenuResponse::Noop(self)
        }
    }

    #[must_use]
    pub fn add_contents(mut self, ui: &mut Ui) -> Response {
        puffin::profile_function!();
        ui.heading("Boundary Conditions");

        enum BottomPanelResponse {
            Noop(Box<Configurator>),
            GenerateMesh(Data<WithBoundaryConditions>),
        }

        let bottom_panel_contents = |ui: &mut Ui, configurator: Box<Configurator>| {
            puffin::profile_scope!("bottom_panel_contents");
            ui.horizontal(|ui| {
                let generate_mesh = ui
                    .button("Next: Mesh Generation ➡")
                    .on_hover_text("Proceed to mesh the shape with the current boundary conditions")
                    .clicked();
                if generate_mesh {
                    BottomPanelResponse::GenerateMesh(configurator.project_data_with_bc())
                } else {
                    BottomPanelResponse::Noop(configurator)
                }
            })
            .inner
        };

        let _glass_fill = if ui.visuals().dark_mode {
            egui::Color32::from_rgba_unmultiplied(18, 18, 26, 220)
        } else {
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 180)
        };

        let response = bottom_panel::show("boundary_conditions_bottom_panel", ui, |ui| {
            ui.horizontal(|ui| {
                super::premium::status_dot(ui, false);
                ui.add_space(4.0);
                bottom_panel_contents(ui, self.configurator)
            }).inner
        });

        self.configurator = match response.inner {
            BottomPanelResponse::Noop(configurator) => configurator,
            BottomPanelResponse::GenerateMesh(pd) => {
                return Response::GenerateMesh(pd);
            }
        };

        // Fixed width: animating width resizes the central plot every frame and fights `data_aspect`,
        // which looks like spurious zoom and can inject bogus drag deltas into the 3D view.
        const BC_SIDEBAR_WIDTH: f32 = 280.0;

        let side_glass_fill = if ui.visuals().dark_mode {
            egui::Color32::from_rgba_unmultiplied(14, 14, 22, 215)
        } else {
            egui::Color32::from_rgba_unmultiplied(250, 252, 255, 220)
        };

        SidePanel::left("boundary_selection_panel")
            .resizable(true)
            .default_width(BC_SIDEBAR_WIDTH)
            .min_width(200.0)
            .frame(egui::Frame::none()
                .fill(side_glass_fill)
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(99, 102, 241, 25)))
                .inner_margin(8.0))
            .show_inside(ui, |ui| self.add_boundary_list(ui));
        CentralPanel::default()
            .frame(Frame::default())
            .show_inside(ui, |ui| self.add_preview(ui));

        macro_rules! error_dialogs {
            ( $( $opt:expr ),* ) => {
                $( if let Some(err) = $opt.as_ref() {
                    if error_dialog::show(err, ui.ctx()).closed() {
                        $opt = None;
                    }
                } )*
            };
        }

        error_dialogs!(self.boundary_state.point_fetch_error, self.input_error);

        let Some(mut state) = self.dialog_state.take() else {
            return Response::Noop(self);
        };
        match dialog::show(&mut state, ui.ctx()) {
            dialog::Response::Noop => {
                self.dialog_state = Some(state);
            }
            dialog::Response::Conditions(result) => match result {
                Ok(condition) => self
                    .configurator
                    .set_condition(self.boundary_state.id, condition),
                Err(err) => {
                    self.input_error = err.into();
                    self.dialog_state = Some(state);
                }
            },
            dialog::Response::Cancel => {}
        }
        Response::Noop(self)
    }

    fn is_3d(&self) -> bool {
        !self.configurator.polygon_data().polyhedron_set().is_empty()
    }

    fn add_boundary_list(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        ui.add_space(10.0);
        if self.is_3d() {
            self.add_face_3d_panel(ui);
            ui.add_space(10.0);
            if ui.button(format!("{} Reset Camera", crate::ui::unicode_symbols::REFRESH)).clicked() {
                self.rotation_x = -0.5;
                self.rotation_y = 0.5;
                self.needs_reset = true;
            }
            return;
        }
        
        // --- Boundary Selection Card ---
        super::premium::premium_card(ui, "🎯 Select Boundary", |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(|ui| {
                ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    if let Some(id) = self.add_boundary_controls_from_polygon_set(ui) {
                        self.boundary_state = Box::new(BoundaryState::new(id, &self.configurator));
                    }
                });
            });
        });

        ui.add_space(12.0);

        // --- Boundary Condition Card ---
        super::premium::premium_card(ui, "⚙ Condition Settings", |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(|ui| {
                let conditions = self
                    .configurator
                    .get_condition(&self.boundary_state.id)
                    .expect("Each id should have conditions mapped");
                
                ui.horizontal(|ui| {
                    ui.label(format!("ID: {}", *self.boundary_state.id.curve_id() + 1));
                    let resp = ui.add(egui::Button::new("➕ Set Conditions").fill(ui.visuals().selection.bg_fill).rounding(egui::Rounding::same(6.0)));
                    if resp.clicked() {
                        self.dialog_state = Some(Box::new(conditions.into()));
                    }
                });

                ui.add_space(8.0);
                ui.label(match conditions {
                    BoundaryCondition::Free => egui::RichText::new("⚪ Free (No constraints)").small(),
                    BoundaryCondition::Force(_) => egui::RichText::new("🟢 Force Applied").small().color(egui::Color32::GREEN),
                    BoundaryCondition::Displacement(_) => egui::RichText::new("🟣 Displacement Fixed").small().color(VIOLET),
                });
                
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);
                
                self.add_split_controls(ui);
            });
        });
    }

    #[must_use]
    fn add_boundary_controls_from_polygon_set(&self, ui: &mut Ui) -> Option<BoundaryId> {
        puffin::profile_function!();
        let mut selected_id = None;
        ui.vertical_centered_justified(|ui| {
            let polygon = &self
                .configurator
                .polygon_data()
                .polygon_set()
                .polygon_with_holes()[0];
            ui.group(|ui| {
                ui.group(|ui| {
                    ui.label("Outer boundaries");
                    if let Some(id) = self.add_boundary_controls(polygon.outer_boundaries(), ui) {
                        selected_id = id.into();
                    }
                });
                polygon.hole_ids().for_each(|hole_id| {
                    ui.group(|ui| {
                        ui.label(format!("Hole {}", *hole_id + 1));
                        if let Some(id) =
                            self.add_boundary_controls(polygon.hole_boundaries(hole_id), ui)
                        {
                            selected_id = id.into();
                        }
                    });
                })
            });
        });
        selected_id
    }

    #[must_use]
    fn add_boundary_controls<'a>(
        &self,
        boundaries: impl Iterator<Item = (BoundaryId, &'a Curve)>,
        ui: &mut Ui,
    ) -> Option<BoundaryId> {
        puffin::profile_function!();
        boundaries
            .map(|(id, _)| (format!("Boundary {}", *id.curve_id() + 1), id))
            .filter_map(|(text, id)| {
                ui.radio(self.boundary_state.id == id, text)
                    .clicked()
                    .then_some(id)
            })
            .last()
    }

    fn split_coordinate<T>(&self) -> T
    where
        T: TryFrom<f64>,
        T::Error: Debug,
    {
        T::try_from(self.boundary_state.split_state.value()).expect("Split state is valid")
    }

    fn split_point(&mut self) -> Option<Point> {
        puffin::profile_function!();
        let polygon_set = self.configurator.polygon_data().polygon_set();
        let curve = polygon_set.polygon_with_holes()[0].boundary_with_id(&self.boundary_state.id);
        let value: Algebraic = self.split_coordinate();
        let result = match curve {
            Curve::Line(line) => match line {
                LineSegment::Horizontal(line) => line.point_at_x(&line.clamp_x(&value)),
                LineSegment::Vertical(line) => line.point_at_y(&line.clamp_y(&value)),
                LineSegment::Oblique(line) => line.point_at_x(&line.clamp_x(&value)),
            },
            Curve::Ellipse(arc) => arc.point_at_x(&arc.clamp_x(&value)),
        };
        match result {
            Ok(point) => Some(point),
            Err(err) => {
                self.boundary_state.point_fetch_error = err.into();
                None
            }
        }
    }

    fn add_split_controls(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        let response = ui.collapsing("Split boundary", |ui| {
            ui.collapsing("Instructions", |ui| {
                ui.label(
                    "Split a boundary at the coordinates given below.\n\
                Slide the slider head or enter a value between 0 and 1. The value is ratio of \
                lengths of sub arc to the total arc (or a segment).\n\
                The point at which a new vertex will be inserted is marked by a cross in the ui.",
                );
            });
            let range = self.boundary_state.split_state.range();
            let prefix = match self.boundary_state.split_state {
                SplitState::X(_) => "x: ",
                SplitState::Y(_) => "y: ",
            };
            ui.add(
                Slider::new(self.boundary_state.split_state.value_mut(), range)
                    .prefix(prefix)
                    .fixed_decimals(2)
                    .trailing_fill(true),
            );
            let coordinate = self.split_coordinate();
            let coordinate = match self.boundary_state.split_state {
                SplitState::X(_) => Coordinate::X(coordinate),
                SplitState::Y(_) => Coordinate::Y(coordinate),
            };
            let point = self.split_point()?;
            let [x, y] = point.into();
            ui.label(format!("Point: {x:.2}, {y:.2}",));
            ui.button("Split")
                .on_hover_text("Split the highlited boundary at the point marked by a cross")
                .clicked()
                .then_some(coordinate)
        });
        self.boundary_state.show_point = response.fully_open();
        let Some(coordinate) = response.body_returned.flatten() else {
            return;
        };
        self.configurator
            .split_curve(self.boundary_state.id, coordinate);
        self.boundary_state = Box::new(BoundaryState::new(
            self.configurator.first_boundary_id(),
            &self.configurator,
        ));
    }

    fn add_preview(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        ui.centered_and_justified(|ui| self.plot_polygon_with_holes(ui));
    }

    fn plot_polygon_with_holes(&mut self, ui: &mut Ui) {
        puffin::profile_function!();
        let is_3d = self.is_3d();
        let mut rotation_x = self.rotation_x;
        let mut rotation_y = self.rotation_y;
        let auto = self.needs_reset;
        if auto { self.needs_reset = false; }

        plot_utils::plot("bc_plot")
            .auto_bounds(egui::Vec2b::new(auto, auto))
            .show_background(false)
            .show_axes(false)
            .show_grid(false)
            .show(ui, |ui| {
            if is_3d {
                if ui.response().dragged_by(egui::PointerButton::Secondary) {
                    let delta = ui.response().drag_delta();
                    rotation_y += delta.x * 0.01;
                    rotation_x += delta.y * 0.01;
                }

                let polygon_data = self.configurator.polygon_data();
                plot_utils::plot_solid_geometry(ui, polygon_data, rotation_x, rotation_y);

                super::gnomon::draw_gnomon(ui, &mut rotation_x, &mut rotation_y);
            } else {
                let transform = |id: BoundaryId, ctx: &Context, line: Line| {
                    let conditions = self
                        .configurator
                        .get_condition(&id)
                        .expect("Boundary id is valid");
                    match conditions {
                        BoundaryCondition::Free => plot_utils::default_transform(id, ctx, line),
                        BoundaryCondition::Force(_) => line.color(Color32::GREEN),
                        BoundaryCondition::Displacement(_) => line.color(VIOLET),
                    }
                };
                let polygon_set = self.configurator.polygon_data().polygon_set();
                plot_utils::plot_polygon_set(ui, polygon_set, transform);
                let polygon = &polygon_set.polygon_with_holes()[0];
                self.plot_polygon_boundary_names(ui, polygon);
                Self::plot_hole_names(ui, polygon);
                Self::plot_vertices(ui, polygon_set);
                if self.boundary_state.show_point {
                    self.plot_split_point(ui);
                }
            }
        });

        self.rotation_x = rotation_x;
        self.rotation_y = rotation_y;
    }

    fn plot_polygon_boundary_names(&self, ui: &mut PlotUi, polygon: &PolygonWithHoles) {
        puffin::profile_function!();
        polygon
            .outer_boundaries()
            .chain(
                polygon
                    .hole_ids()
                    .flat_map(|hole_id| polygon.hole_boundaries(hole_id)),
            )
            .for_each(|(id, curve)| {
                let text = RichText::new(format!("B{}", *id.curve_id() + 1))
                    .heading()
                    .strong();
                let [x, y] = curve.mid_point().into();
                ui.text(Text::new(
                    PlotPoint::new(x, y),
                    if self.boundary_state.id == id {
                        text.color(if super::is_dark_mode(ui.ctx()) {
                            Color32::LIGHT_RED
                        } else {
                            Color32::DARK_RED
                        })
                    } else {
                        text
                    },
                ))
            });
    }

    fn plot_hole_names(ui: &mut PlotUi, polygon: &PolygonWithHoles) {
        puffin::profile_function!();
        polygon.hole_ids().enumerate().for_each(|(index, hole_id)| {
            let [x, y] = polygon.hole_with_id(hole_id).centroid().into();
            ui.text(Text::new(
                PlotPoint::new(x, y),
                RichText::new(format!("H{}", index + 1)).heading().weak(),
            ))
        });
    }

    fn plot_vertices(ui: &mut PlotUi, polygon_set: &PolygonSet) {
        puffin::profile_function!();
        ui.points(
            Points::new(
                polygon_set
                    .vertices()
                    .map(Into::into)
                    .collect::<Vec<[f64; 2]>>(),
            )
            .radius(4.0)
            .color(super::on_primary_color(ui.ctx()))
            .shape(MarkerShape::Diamond),
        );
    }

    fn plot_split_point(&mut self, ui: &mut PlotUi) {
        puffin::profile_function!();
        let Some(point) = self.split_point().map(Into::into) else {
            return;
        };
        ui.points(
            Points::new(vec![point])
                .shape(MarkerShape::Cross)
                .radius(6.0)
                .color(super::on_primary_color(ui.ctx()))
                .highlight(true),
        );
    }

    fn add_face_3d_panel(&mut self, ui: &mut Ui) {
        ui.label(
            egui::RichText::new("3D Face Boundary Conditions")
                .heading()
                .strong(),
        );
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(
                "Each rule applies a condition to all mesh nodes where the chosen axis coordinate satisfies the comparison. Use this to constrain faces of 3D shapes.",
            )
            .small()
            .weak(),
        );
        ui.add_space(10.0);

        // --- Existing rules ---
        let conditions = self.configurator.face_3d_conditions().clone();
        if conditions.is_empty() {
            ui.label(egui::RichText::new("No face conditions defined yet.").small().weak());
        } else {
            ui.label(egui::RichText::new("Active Rules:").strong());
            ScrollArea::vertical()
                .id_salt("face3d_list")
                .max_height(180.0)
                .show(ui, |ui| {
                    let mut to_remove: Option<usize> = None;
                    for (i, fc) in conditions.iter().enumerate() {
                        ui.horizontal(|ui| {
                            let cond_tag = match &fc.condition {
                                BoundaryCondition::Free => "Free",
                                BoundaryCondition::Force(_) => "Force",
                                BoundaryCondition::Displacement(_) => "Fixed",
                            };
                            ui.label(egui::RichText::new(
                                format!("[{}] {}", cond_tag, fc.label())
                            ).small());
                            if ui.small_button("🗑").clicked() {
                                to_remove = Some(i);
                            }
                        });
                    }
                    if let Some(idx) = to_remove {
                        self.configurator.face_3d_conditions_mut().remove(idx);
                    }
                });
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(6.0);
        ui.label(egui::RichText::new("New Rule").strong());
        ui.add_space(4.0);

        // Axis
        ui.horizontal(|ui| {
            ui.label("Axis:");
            for axis in Axis3D::iter() {
                ui.radio_value(&mut self.pending_face.axis, axis, axis.to_string());
            }
        });

        // Comparison
        ui.horizontal_wrapped(|ui| {
            ui.label("Rule:");
            for cmp in PlaneComparison::iter() {
                ui.radio_value(&mut self.pending_face.comparison, cmp, cmp.to_string());
            }
        });

        // Threshold value
        ui.horizontal(|ui| {
            ui.label("Threshold:");
            ui.add(
                egui::DragValue::new(&mut self.pending_face.value)
                    .speed(0.01)
                    .fixed_decimals(4),
            );
        });

        if let Some((min_b, max_b)) = self
            .configurator
            .polygon_data()
            .polyhedron_vertex_axis_bounds()
        {
            let axis_i = match self.pending_face.axis {
                Axis3D::X => 0usize,
                Axis3D::Y => 1,
                Axis3D::Z => 2,
            };
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("Snap to shape:").small());
                if ui
                    .small_button("Min (≤)")
                    .on_hover_text(
                        "Use the lowest coordinate on the selected axis — sets rule to ≤ (min face).",
                    )
                    .clicked()
                {
                    self.pending_face.comparison = PlaneComparison::LessOrEqual;
                    self.pending_face.value = min_b[axis_i];
                }
                if ui
                    .small_button("Max (≥)")
                    .on_hover_text(
                        "Use the highest coordinate on the selected axis — sets rule to ≥ (max face).",
                    )
                    .clicked()
                {
                    self.pending_face.comparison = PlaneComparison::GreaterOrEqual;
                    self.pending_face.value = max_b[axis_i];
                }
                if ui
                    .small_button("Mid (≈)")
                    .on_hover_text(
                        "Use the midpoint on this axis with approximate equality; ε set to ~1% of extent.",
                    )
                    .clicked()
                {
                    self.pending_face.comparison = PlaneComparison::Approx;
                    self.pending_face.value = 0.5 * (min_b[axis_i] + max_b[axis_i]);
                    let extent = (max_b[axis_i] - min_b[axis_i]).abs();
                    self.pending_face.epsilon = (extent * 0.01).max(1e-9);
                }
            });
        }

        // Tolerance (only for Approx)
        if matches!(self.pending_face.comparison, PlaneComparison::Approx) {
            ui.horizontal(|ui| {
                ui.label("Tolerance ε:");
                ui.add(
                    egui::DragValue::new(&mut self.pending_face.epsilon)
                        .speed(0.0001)
                        .fixed_decimals(6),
                );
            });
        }

        // Condition type
        ui.add_space(4.0);
        ui.label(egui::RichText::new("Apply:").strong());
        let is_free = matches!(self.pending_face.condition, BoundaryCondition::Free);
        let is_fixed = matches!(self.pending_face.condition, BoundaryCondition::Displacement(_));
        ui.horizontal(|ui| {
            if ui.radio(is_free, "Free").clicked() {
                self.pending_face.condition = BoundaryCondition::Free;
            }
            if ui.radio(is_fixed, "Fixed (zero displacement)").clicked() {
                use cpd::boundary_condition::Displacement;
                use function::{Function, piecewise_linear::{Piece, PiecewiseLinear}};
                use nalgebra::Vector3;
                let z = || Function::Piecewise(PiecewiseLinear::builder()
                    .piece(Piece::builder().end_value(0.0).width(1.0e9).build())
                    .build());
                self.pending_face.condition = BoundaryCondition::Displacement(
                    Displacement::XYZ(Vector3::new(z(), z(), z())),
                );
            }
        });

        ui.add_space(8.0);
        if ui
            .add(
                egui::Button::new("➕ Add Rule")
                    .fill(ui.visuals().selection.bg_fill)
                    .rounding(egui::Rounding::same(6.0)),
            )
            .clicked()
        {
            let rule = self.pending_face.clone();
            self.configurator.face_3d_conditions_mut().push(rule);
        }
    }
}

mod serde_impl {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    impl Serialize for Page {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.configurator
                .project_data_with_bc_cloned()
                .serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Page {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Data::<WithBoundaryConditions>::deserialize(deserializer).map(Page::from)
        }
    }
}
