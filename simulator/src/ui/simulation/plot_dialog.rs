use crate::{
    model::{engine::PlotItems, PolygonData},
    ui::{self, always_open_window::AlwaysOpenWindow, dialog_utils::ok_cancel, plot_utils},
};
use cgal::{BoundaryId, PolygonWithHoles};
use ecolor::Color32;
use egui::{Align, Context, Layout, RichText, Separator, Ui};
use egui_plot::{PlotPoint, PlotUi, Text};

const DIALOG_SIZE: f32 = 600.0;

#[derive(Debug)]
pub struct State {
    plot_items: PlotItems,
    polygon_data: PolygonData,
    selected_boundary_id: BoundaryId,
}

impl State {
    pub fn new(plot_items: PlotItems, polygon_data: PolygonData) -> Self {
        puffin::profile_function!();
        Self {
            selected_boundary_id: plot_items
                .free()
                .iter()
                .chain(plot_items.free().iter())
                .chain(plot_items.displacement().iter())
                .copied()
                .next()
                .expect("State cannot be created with empty plot items"),
            plot_items,
            polygon_data,
        }
    }
}

pub enum Response {
    Noop,
    Cancel,
    BoundaryId(BoundaryId),
}

pub fn show(ctx: &Context, state: &mut State) -> Response {
    puffin::profile_function!();
    AlwaysOpenWindow::new("Select boundary")
        .resizable(false)
        .default_width(DIALOG_SIZE)
        .default_height(DIALOG_SIZE)
        .show(ctx, |ui| {
            puffin::profile_scope!("plot_boundary_select_window");
            ui.horizontal(|ui| {
                ui.vertical(|ui| show_side_controls(ui, state));
                ui.add(Separator::default().vertical().grow(DIALOG_SIZE));
                ui.vertical(|ui| show_plot(ui, state)).inner
            })
            .inner
        })
}

fn boundary_id_label(id: &BoundaryId) -> String {
    match id {
        BoundaryId::OuterBoundary(curve_id) => format!("Outer boundary B{}", **curve_id + 1),
        BoundaryId::Hole(hole_id, curve_id) => {
            format!("Hole ({}) boundary B{}", **hole_id + 1, **curve_id + 1)
        }
    }
}

fn show_side_controls(ui: &mut Ui, state: &mut State) {
    puffin::profile_function!();
    macro_rules! add_controls {
        ( $item:ident, $kind:literal ) => {
            if !state.plot_items.$item().is_empty() {
                ui.group(|ui| {
                    ui.label(const_format::formatcp!("{} boundaries", $kind));
                    state.plot_items.$item().iter().copied().for_each(|id| {
                        ui.radio_value(&mut state.selected_boundary_id, id, boundary_id_label(&id));
                    })
                });
            }
        };
    }
    add_controls!(free, "Free");
    add_controls!(force, "Force");
    add_controls!(displacement, "Displacement");
}

fn show_plot(ui: &mut Ui, state: &State) -> Response {
    puffin::profile_function!();
    ui.vertical_centered_justified(|ui| {
        plot_utils::plot_without_clutter("plot_boundary_select_window_plot")
            .data_aspect(1.0)
            .width(ui.available_width())
            .view_aspect(1.0)
            .show(ui, |ui| {
                let polygon_set = state.polygon_data.polygon_set();
                plot_utils::plot_polygon_set(ui, polygon_set, plot_utils::default_transform);
                plot_polygon_boundary_names(ui, &polygon_set.polygon_with_holes()[0], state)
            });
        ui.with_layout(
            Layout::right_to_left(Align::Min),
            |ui| match ok_cancel::buttons(ui) {
                ok_cancel::Response::Ok => Response::BoundaryId(state.selected_boundary_id),
                ok_cancel::Response::Cancel => Response::Cancel,
                ok_cancel::Response::Noop => Response::Noop,
            },
        )
        .inner
    })
    .inner
}

fn plot_polygon_boundary_names(ui: &mut PlotUi, polygon: &PolygonWithHoles, state: &State) {
    puffin::profile_function!();
    polygon
        .outer_boundaries()
        .chain(
            polygon
                .hole_ids()
                .flat_map(|hole_id| polygon.hole_boundaries(hole_id)),
        )
        .filter(|(id, _)| state.plot_items.contains_id(id))
        .for_each(|(id, curve)| {
            let text = RichText::new(format!("B{}", *id.curve_id() + 1))
                .heading()
                .strong();
            let [x, y] = curve.mid_point().into();
            ui.text(Text::new(
                PlotPoint::new(x, y),
                if state.selected_boundary_id == id {
                    text.color(if ui::is_dark_mode(ui.ctx()) {
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
