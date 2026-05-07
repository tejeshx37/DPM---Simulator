use crate::ui::{always_open_window::AlwaysOpenWindow, dialog_utils::ok_cancel};
use cpd::boundary_condition::{BoundaryCondition, Displacement};
use egui::{Align, Context, Layout, Ui};
use function::{
    piecewise_linear::{Piece, PiecewiseLinear},
    Function,
};
use nalgebra::{Vector2, Vector3};
use std::mem;
use strum::{Display, EnumIter, IntoEnumIterator};

#[derive(Debug, Default, Clone)]
struct PieceInput {
    end_value: String,
    duration: String,
}

impl TryFrom<&PieceInput> for Piece {
    type Error = String;

    fn try_from(value: &PieceInput) -> Result<Self, Self::Error> {
        macro_rules! parse {
            ( $field:ident ) => {
                value
                    .$field
                    .parse()
                    .map_err(|_| const_format::formatcp!("{} is invalid", stringify!($field)))
            };
        }
        let piece = Piece::builder()
            .end_value(parse!(end_value)?)
            .width(parse!(duration)?)
            .build();
        Ok(piece)
    }
}

impl From<&Piece> for PieceInput {
    fn from(value: &Piece) -> Self {
        Self {
            end_value: value.end_value().to_string(),
            duration: value.width().to_string(),
        }
    }
}

#[derive(Debug, Default, Clone)]
struct PiecewiseLinearInput {
    pieces: Vec<PieceInput>,
}

impl TryFrom<&PiecewiseLinearInput> for PiecewiseLinear {
    type Error = String;

    fn try_from(value: &PiecewiseLinearInput) -> Result<Self, Self::Error> {
        value
            .pieces
            .iter()
            .try_fold(Self::builder(), |builder, piece| {
                piece.try_into().map(|piece| builder.piece(piece))
            })
            .map(|builder| builder.build())
    }
}

impl From<&PiecewiseLinear> for PiecewiseLinearInput {
    fn from(value: &PiecewiseLinear) -> Self {
        Self {
            pieces: value.pieces().iter().map(PieceInput::from).collect(),
        }
    }
}

impl From<&Function> for PiecewiseLinearInput {
    fn from(value: &Function) -> Self {
        match value {
            Function::Piecewise(f) => f.into(),
            Function::Superposed(_) => unimplemented!(),
        }
    }
}

impl TryFrom<&PiecewiseLinearInput> for Function {
    type Error = String;

    fn try_from(value: &PiecewiseLinearInput) -> Result<Self, Self::Error> {
        value.try_into().map(Self::Piecewise)
    }
}

#[derive(Debug, Default, Clone)]
struct VectorInput {
    x: PiecewiseLinearInput,
    y: PiecewiseLinearInput,
    z: PiecewiseLinearInput,
}

impl From<&Vector3<Function>> for VectorInput {
    fn from(value: &Vector3<Function>) -> Self {
        Self {
            x: (&value.x).into(),
            y: (&value.y).into(),
            z: (&value.z).into(),
        }
    }
}

impl TryFrom<&VectorInput> for Vector3<Function> {
    type Error = String;

    fn try_from(value: &VectorInput) -> Result<Self, Self::Error> {
        Ok(Self::new(
            (&value.x).try_into()?,
            (&value.y).try_into()?,
            (&value.z).try_into()?,
        ))
    }
}

#[derive(Debug, Default, Clone)]
struct Vector2Input {
    x: PiecewiseLinearInput,
    y: PiecewiseLinearInput,
}

impl From<&Vector2<Function>> for Vector2Input {
    fn from(value: &Vector2<Function>) -> Self {
        Self {
            x: (&value.x).into(),
            y: (&value.y).into(),
        }
    }
}

impl TryFrom<&Vector2Input> for Vector2<Function> {
    type Error = String;

    fn try_from(value: &Vector2Input) -> Result<Self, Self::Error> {
        Ok(Self::new(
            (&value.x).try_into()?,
            (&value.y).try_into()?,
        ))
    }
}

#[derive(Debug, Clone, EnumIter, Display)]
enum DisplacementInput {
    X(PiecewiseLinearInput),
    Y(PiecewiseLinearInput),
    Z(PiecewiseLinearInput),
    XY(Vector2Input),
    YZ(Vector2Input),
    XZ(Vector2Input),
    XYZ(VectorInput),
}

impl PartialEq for DisplacementInput {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}

impl Default for DisplacementInput {
    fn default() -> Self {
        Self::XYZ(VectorInput::default())
    }
}

impl From<&Displacement> for DisplacementInput {
    fn from(value: &Displacement) -> Self {
        match value {
            Displacement::X(x) => Self::X(x.into()),
            Displacement::Y(y) => Self::Y(y.into()),
            Displacement::Z(z) => Self::Z(z.into()),
            Displacement::XY(v) => Self::XY(v.into()),
            Displacement::YZ(v) => Self::YZ(v.into()),
            Displacement::XZ(v) => Self::XZ(v.into()),
            Displacement::XYZ(v) => Self::XYZ(v.into()),
        }
    }
}

impl TryFrom<&DisplacementInput> for Displacement {
    type Error = String;

    fn try_from(value: &DisplacementInput) -> Result<Self, Self::Error> {
        Ok(match value {
            DisplacementInput::X(x) => Self::X(x.try_into()?),
            DisplacementInput::Y(y) => Self::Y(y.try_into()?),
            DisplacementInput::Z(z) => Self::Z(z.try_into()?),
            DisplacementInput::XY(v) => Self::XY(v.try_into()?),
            DisplacementInput::YZ(v) => Self::YZ(v.try_into()?),
            DisplacementInput::XZ(v) => Self::XZ(v.try_into()?),
            DisplacementInput::XYZ(v) => Self::XYZ(v.try_into()?),
        })
    }
}

#[derive(Debug, Default, Clone, Display, EnumIter)]
enum BoundaryConditionInput {
    #[default]
    Free,
    Force(VectorInput),
    Displacement(DisplacementInput),
}

impl PartialEq for BoundaryConditionInput {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}

impl From<&BoundaryCondition> for BoundaryConditionInput {
    fn from(value: &BoundaryCondition) -> Self {
        match value {
            BoundaryCondition::Free => Self::Free,
            BoundaryCondition::Force(force) => Self::Force(force.into()),
            BoundaryCondition::Displacement(displacement) => {
                Self::Displacement(displacement.into())
            }
        }
    }
}

impl TryFrom<&BoundaryConditionInput> for BoundaryCondition {
    type Error = String;

    fn try_from(value: &BoundaryConditionInput) -> Result<Self, Self::Error> {
        match value {
            BoundaryConditionInput::Free => Ok(Self::Free),
            BoundaryConditionInput::Force(force) => force.try_into().map(Self::Force),
            BoundaryConditionInput::Displacement(displacement) => {
                displacement.try_into().map(Self::Displacement)
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct State {
    boundary_condition_input: BoundaryConditionInput,
}

impl From<&BoundaryCondition> for State {
    fn from(value: &BoundaryCondition) -> Self {
        Self {
            boundary_condition_input: value.into(),
        }
    }
}

pub enum Response {
    Noop,
    Conditions(Result<BoundaryCondition, String>),
    Cancel,
}

pub fn show(state: &mut State, ctx: &Context) -> Response {
    AlwaysOpenWindow::new("Set boundary conditions")
        .resizable(false)
        .default_width(340.0)
        .show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.horizontal(|ui| {
                    BoundaryConditionInput::iter().for_each(|condition| {
                        let text = condition.to_string();
                        ui.radio_value(&mut state.boundary_condition_input, condition, text);
                    });
                });
                match &mut state.boundary_condition_input {
                    BoundaryConditionInput::Free => {
                        ui.label("There are no constraints imposed on this boundary.");
                    }
                    BoundaryConditionInput::Force(force_input) => {
                        force_input_layout(force_input, ui)
                    }
                    BoundaryConditionInput::Displacement(displacement_input) => {
                        displacement_input_layout(displacement_input, ui)
                    }
                }
                ui.with_layout(Layout::right_to_left(Align::Min), |buttons_ui| {
                    match ok_cancel::buttons(buttons_ui) {
                        ok_cancel::Response::Ok => {
                            Response::Conditions((&state.boundary_condition_input).try_into())
                        }
                        ok_cancel::Response::Cancel => Response::Cancel,
                        ok_cancel::Response::Noop => Response::Noop,
                    }
                })
                .inner
            })
            .inner
        })
}

fn piece_input_layout(input: &mut PieceInput, ui: &mut Ui) -> bool {
    ui.horizontal(|ui| {
        ui.label("End value");
        ui.text_edit_singleline(&mut input.end_value);
    });
    ui.horizontal(|ui| {
        ui.label("Duration");
        ui.text_edit_singleline(&mut input.duration);
    });
    ui.button("Remove piece").clicked()
}

fn piecewise_function_instructions(ui: &mut Ui) {
    ui.label("Each 'piece' of this function has an end value and a duration associated with it.\n\
    First piece always starts from 0.\n\
    The value at each time step is then linearly interpolated between start and end value of a piece.\n\
    You may choose to not provide any pieces, rendering it as basically an absent component.\n\
    If the sum of durations of each piece is lesser than the total duration of the simulation,\
    then the condition will not be applied for the remaining duration.",
    );
}

fn piecewise_function_input_layout(input: &mut PiecewiseLinearInput, ui: &mut Ui) {
    let pieces = mem::take(&mut input.pieces);
    input
        .pieces
        .extend(pieces.into_iter().filter_map(|mut piece| {
            ui.group(|ui| !piece_input_layout(&mut piece, ui))
                .inner
                .then_some(piece)
        }));
    if ui.button("Add new piece").clicked() {
        input.pieces.push(PieceInput::default());
    }
}

fn vector_input_instructions(ui: &mut Ui) {
    ui.label(
        "X, Y, and Z components can be set independently.\n\
    Each component is considered as a piece-wise defined function.",
    );
}

fn vector_input_layout(vector_input: &mut VectorInput, ui: &mut Ui) {
    ui.group(|ui| {
        ui.heading("X Component");
        piecewise_function_input_layout(&mut vector_input.x, ui);
    });
    ui.separator();
    ui.group(|ui| {
        ui.heading("Y Component");
        piecewise_function_input_layout(&mut vector_input.y, ui);
    });
    ui.separator();
    ui.group(|ui| {
        ui.heading("Z Component");
        piecewise_function_input_layout(&mut vector_input.z, ui);
    });
}

fn vector2_input_layout(vector_input: &mut Vector2Input, ui: &mut Ui) {
    ui.group(|ui| {
        ui.heading("Component 1");
        piecewise_function_input_layout(&mut vector_input.x, ui);
    });
    ui.separator();
    ui.group(|ui| {
        ui.heading("Component 2");
        piecewise_function_input_layout(&mut vector_input.y, ui);
    });
}

fn force_input_layout(vector_input: &mut VectorInput, ui: &mut Ui) {
    ui.collapsing("Instructions", |ui| {
        vector_input_instructions(ui);
        piecewise_function_instructions(ui);
    });
    vector_input_layout(vector_input, ui);
}

fn displacement_input_layout(displacement_input: &mut DisplacementInput, ui: &mut Ui) {
    ui.collapsing("Instructions", |ui| {
        ui.label(
            "Selecting just X or Y component will leave the other component unconstrained, i.e.,\
        it'll be treated just as if it was that of a free boundary",
        );
        if matches!(displacement_input, DisplacementInput::XY(_)) {
            vector_input_instructions(ui);
        }
        piecewise_function_instructions(ui);
    });
    ui.horizontal(|ui| {
        DisplacementInput::iter().for_each(|input| {
            let text = input.to_string();
            ui.radio_value(displacement_input, input, text);
        });
    });
    match displacement_input {
        DisplacementInput::X(input) | DisplacementInput::Y(input) | DisplacementInput::Z(input) => {
            piecewise_function_input_layout(input, ui)
        }
        DisplacementInput::XY(input) | DisplacementInput::YZ(input) | DisplacementInput::XZ(input) => {
            vector2_input_layout(input, ui)
        }
        DisplacementInput::XYZ(input) => vector_input_layout(input, ui),
    }
}
