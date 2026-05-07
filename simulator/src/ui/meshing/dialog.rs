use crate::ui::{always_open_window::AlwaysOpenWindow, dialog_utils::ok_cancel};
use egui::{Align, Context, Layout, Ui};

const INPUT_SECTION_MARGIN: f32 = 8.0;

#[derive(Debug)]
pub struct State {
    num_input: String,
    override_size_bound: bool,
    size_bound_input: String,
    thickness_input: String,
}

impl Default for State {
    fn default() -> Self {
        Self {
            num_input: String::from("512"),
            override_size_bound: false,
            size_bound_input: String::from("0.5"),
            thickness_input: String::from("10.0"),
        }
    }
}

pub struct Data {
    pub num_points: u32,
    pub size_bound_override: Option<f64>,
    pub thickness: f64,
}

impl TryFrom<&State> for Data {
    type Error = String;

    fn try_from(value: &State) -> Result<Self, Self::Error> {
        let num_points = value
            .num_input
            .parse()
            .map_err(|_| format!("Invalid point count input {}", value.num_input))?;
        if num_points == 0 {
            return Err(String::from("Number of points must be greater than 0"));
        }
        if num_points > 10000 {
            return Err(String::from("Point count is too high (max 10,000)"));
        }

        let size_bound_override = value
            .override_size_bound
            .then(|| {
                let size_bound: f64 = value
                    .size_bound_input
                    .parse()
                    .map_err(|_| format!("Invalid size bound input {}", value.size_bound_input))?;
                if size_bound < 0.0 {
                    Err(String::from("Size bound should be positive"))
                } else {
                    Ok(size_bound)
                }
            })
            .transpose()?;

        let thickness = value
            .thickness_input
            .parse()
            .map_err(|_| format!("Invalid thickness input {}", value.thickness_input))?;
        if thickness < 0.0 {
            return Err(String::from("Thickness should be positive"));
        }

        Ok(Data {
            num_points,
            size_bound_override,
            thickness,
        })
    }
}

pub enum Response {
    Noop,
    DataResult(Result<Data, String>),
    Cancel,
}

pub fn show(state: &mut State, ctx: &Context) -> Response {
    AlwaysOpenWindow::new("Mesh config")
        .resizable(false)
        .default_width(340.0)
        .show(ctx, |ui| {
            instructions(ui);
            ui.checkbox(&mut state.override_size_bound, "Override size bound");
            ui.group(|ui| input_table_layout(state, ui));
            ui.add_space(INPUT_SECTION_MARGIN);
            ui.with_layout(
                Layout::right_to_left(Align::Min),
                |ui| match ok_cancel::buttons(ui) {
                    ok_cancel::Response::Ok => Response::DataResult(Data::try_from(&*state)),
                    ok_cancel::Response::Cancel => Response::Cancel,
                    ok_cancel::Response::Noop => Response::Noop,
                },
            )
            .inner
        })
}

fn instructions(ui: &mut Ui) {
    ui.collapsing("Instructions", |ui| {
        ui.label(
            "Size bound is the length of the largest edge out of all triangles. \
        By default it is set to the euclidian distance between two adjancent generated points \
        on the boundary.\n\
        Count is the number of points that will be generated on all of the boundaries.\n\
        Thickness is the 3D extrusion depth of the generated 2D mesh into a 3D volume.\n\
        Rest of the interior points will be generated according to the size bound.",
        );
    });
}

fn input_table_layout(state: &mut State, ui: &mut Ui) {
    ui.vertical_centered_justified(|ui| {
        ui.horizontal(|ui| {
            ui.label("Count");
            ui.text_edit_singleline(&mut state.num_input);
        });
        ui.horizontal(|ui| {
            ui.label("Thickness");
            ui.text_edit_singleline(&mut state.thickness_input);
        });
        if !state.override_size_bound {
            return;
        }
        ui.horizontal(|ui| {
            ui.label("Size bound");
            ui.text_edit_singleline(&mut state.size_bound_input);
        });
    });
}
