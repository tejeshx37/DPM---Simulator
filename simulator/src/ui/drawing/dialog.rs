use super::shape::Shape;
use crate::{
    model::shape_configurator::polygon_vertices_parser::{
        Data as PVData, Output as PVOutput, PolygonVerticesParser as PVParser,
    },
    ui::{
        dialog_utils,
        error_dialog,
    },
};
use cgal::{num::Rational, PolygonSetInputKind, RationalPoint};
use egui::{Align, Button, Layout, ScrollArea, Ui};
use rfd::FileDialog;
use std::{
    cell::OnceCell,
    rc::Rc,
    slice::IterMut,
    sync::mpsc::{self, Receiver, TryRecvError},
};

const INPUT_SECTION_MARGIN: f32 = 8.0;

#[derive(Debug)]
struct RationalPointInput {
    x: String,
    y: String,
}

impl Default for RationalPointInput {
    fn default() -> Self {
        Self {
            x: String::from("0"),
            y: String::from("0"),
        }
    }
}

impl From<&RationalPoint> for RationalPointInput {
    fn from(value: &RationalPoint) -> Self {
        Self {
            x: value.x.to_string(),
            y: value.y.to_string(),
        }
    }
}

#[derive(Debug)]
struct RationalPoint3Input {
    x: String,
    y: String,
    z: String,
}

impl Default for RationalPoint3Input {
    fn default() -> Self {
        Self {
            x: String::from("0"),
            y: String::from("0"),
            z: String::from("0"),
        }
    }
}

impl TryInto<cgal::RationalPoint3> for &RationalPoint3Input {
    type Error = String;

    fn try_into(self) -> Result<cgal::RationalPoint3, Self::Error> {
        self.x.parse().and_then(|x: Rational| {
            self.y.parse().and_then(|y| {
                self.z.parse().map(|z| cgal::RationalPoint3::new(x, y, z))
            })
        })
    }
}

impl TryInto<RationalPoint> for &RationalPointInput {
    type Error = String;

    fn try_into(self) -> Result<RationalPoint, Self::Error> {
        self.x
            .parse()
            .and_then(|x: Rational| self.y.parse().map(|y| RationalPoint::new(x, y)))
    }
}

#[derive(Debug, Default)]
struct RectInputState {
    top_left: RationalPointInput,
    bottom_right: RationalPointInput,
}

impl TryInto<PolygonSetInputKind> for &RectInputState {
    type Error = String;

    fn try_into(self) -> Result<PolygonSetInputKind, Self::Error> {
        macro_rules! parse {
            ( $point:ident, $coord:ident ) => {
                self.$point.$coord.parse::<f64>().map_err(|_| {
                    format!(
                        "Invalid {} coordinate '{}' of point {}",
                        stringify!($coord),
                        self.$point.$coord,
                        stringify!($point).trim_matches('_'),
                    )
                })
            };
        }

        let tl_x = parse!(top_left, x)?;
        let br_x = parse!(bottom_right, x)?;
        if tl_x >= br_x {
            return Err(String::from(
                "Top left cannot be to the right of bottom right",
            ));
        }

        let tl_y = parse!(top_left, y)?;
        let br_y = parse!(bottom_right, y)?;
        if tl_y <= br_y {
            return Err(String::from("Top left cannot be below bottom right"));
        }

        let top_right =
            self.bottom_right.x.parse().and_then(|x: Rational| {
                self.top_left.y.parse().map(|y| RationalPoint::new(x, y))
            })?;
        let bottom_left = self.top_left.x.parse().and_then(|x: Rational| {
            self.bottom_right
                .y
                .parse()
                .map(|y| RationalPoint::new(x, y))
        })?;
        let bottom_right = (&self.bottom_right).try_into()?;
        let top_left = (&self.top_left).try_into()?;

        Ok(PolygonSetInputKind::LinearPolygon(vec![
            bottom_left,
            bottom_right,
            top_right,
            top_left,
        ]))
    }
}

#[derive(Debug, Default)]
struct PolygonInputState {
    vertices: PolygonVertices,
    parser: Rc<OnceCell<PVParser>>,
    output_receiver: Option<Receiver<PVOutput>>,
    err: Option<String>,
}

impl PolygonInputState {
    fn parser(&self) -> &PVParser {
        self.parser.get_or_init(PVParser::default)
    }
}

#[derive(Debug)]
struct PolygonVertices {
    points: Vec<RationalPointInput>,
}

impl PolygonVertices {
    // Triangle is the polygon with least sides
    const MIN_SIDES: usize = 3;

    fn push_default(&mut self) {
        self.points.push(Default::default());
    }

    fn can_pop(&self) -> bool {
        self.points.len() > Self::MIN_SIDES
    }

    fn pop(&mut self) {
        if self.can_pop() {
            self.points.pop();
        }
    }

    fn iter_mut(&mut self) -> IterMut<'_, RationalPointInput> {
        self.points.iter_mut()
    }
}

impl Default for PolygonVertices {
    fn default() -> Self {
        Self {
            points: (0..Self::MIN_SIDES)
                .map(|_| RationalPointInput::default())
                .collect(),
        }
    }
}

impl TryInto<PolygonSetInputKind> for &PolygonInputState {
    type Error = String;

    fn try_into(self) -> Result<PolygonSetInputKind, Self::Error> {
        let points = &self.vertices.points;
        let points_len = points.len();
        points
            .iter()
            .enumerate()
            .try_fold(Vec::with_capacity(points_len), |mut vec, (idx, input)| {
                if input.x.is_empty() {
                    Err(format!("X coordinate of point {} is empty", idx + 1))
                } else if input.y.is_empty() {
                    Err(format!("Y coordinate of point {} is empty", idx + 1))
                } else {
                    vec.push(input.try_into()?);
                    Ok(vec)
                }
            })
            .map(PolygonSetInputKind::LinearPolygon)
    }
}

#[derive(Debug)]
struct CircleInputState {
    center: RationalPointInput,
    diameter: String,
}

impl Default for CircleInputState {
    fn default() -> Self {
        Self {
            center: Default::default(),
            diameter: String::from('0'),
        }
    }
}

impl TryInto<PolygonSetInputKind> for &CircleInputState {
    type Error = String;

    fn try_into(self) -> Result<PolygonSetInputKind, Self::Error> {
        if self.diameter.is_empty() {
            return Err(String::from("Diameter is empty"));
        }
        Ok(PolygonSetInputKind::Circle {
            center: (&self.center).try_into()?,
            diameter: self.diameter.parse()?,
        })
    }
}

#[derive(Debug)]
struct EllipseInputState {
    center: RationalPointInput,
    width: String,
    height: String,
}

impl Default for EllipseInputState {
    fn default() -> Self {
        Self {
            center: Default::default(),
            width: String::from('0'),
            height: String::from('0'),
        }
    }
}

impl TryInto<PolygonSetInputKind> for &EllipseInputState {
    type Error = String;

    fn try_into(self) -> Result<PolygonSetInputKind, Self::Error> {
        if self.width.is_empty() {
            return Err(String::from("Width is empty"));
        }

        if self.height.is_empty() {
            return Err(String::from("Height is empty"));
        }

        Ok(PolygonSetInputKind::Ellipse {
            center: (&self.center).try_into()?,
            width: self.width.parse()?,
            height: self.height.parse()?,
        })
    }
}

#[derive(Debug)]
enum InputState {
    Rect(RectInputState),
    Polygon(PolygonInputState),
    Circle(CircleInputState),
    Ellipse(EllipseInputState),
    Cube(CubeInputState),
    Cuboid(CuboidInputState),
    Cone(ConeInputState),
    Cylinder(CylinderInputState),
    Sphere(SphereInputState),
}

#[derive(Debug, Default)]
struct CubeInputState {
    center: RationalPoint3Input,
    side_length: String,
}

impl TryInto<PolygonSetInputKind> for &CubeInputState {
    type Error = String;
    fn try_into(self) -> Result<PolygonSetInputKind, Self::Error> {
        Ok(PolygonSetInputKind::Cube {
            center: (&self.center).try_into()?,
            side_length: self.side_length.parse()?,
        })
    }
}

#[derive(Debug, Default)]
struct CuboidInputState {
    center: RationalPoint3Input,
    width: String,
    height: String,
    depth: String,
}

impl TryInto<PolygonSetInputKind> for &CuboidInputState {
    type Error = String;
    fn try_into(self) -> Result<PolygonSetInputKind, Self::Error> {
        Ok(PolygonSetInputKind::Cuboid {
            center: (&self.center).try_into()?,
            width: self.width.parse()?,
            height: self.height.parse()?,
            depth: self.depth.parse()?,
        })
    }
}

#[derive(Debug, Default)]
struct ConeInputState {
    center: RationalPoint3Input,
    radius: String,
    height: String,
}

impl TryInto<PolygonSetInputKind> for &ConeInputState {
    type Error = String;
    fn try_into(self) -> Result<PolygonSetInputKind, Self::Error> {
        Ok(PolygonSetInputKind::Cone {
            center: (&self.center).try_into()?,
            radius: self.radius.parse()?,
            height: self.height.parse()?,
        })
    }
}

#[derive(Debug, Default)]
struct CylinderInputState {
    center: RationalPoint3Input,
    radius: String,
    height: String,
}

impl TryInto<PolygonSetInputKind> for &CylinderInputState {
    type Error = String;
    fn try_into(self) -> Result<PolygonSetInputKind, Self::Error> {
        Ok(PolygonSetInputKind::Cylinder {
            center: (&self.center).try_into()?,
            radius: self.radius.parse()?,
            height: self.height.parse()?,
        })
    }
}

#[derive(Debug, Default)]
struct SphereInputState {
    center: RationalPoint3Input,
    radius: String,
}

impl TryInto<PolygonSetInputKind> for &SphereInputState {
    type Error = String;
    fn try_into(self) -> Result<PolygonSetInputKind, Self::Error> {
        Ok(PolygonSetInputKind::Sphere {
            center: (&self.center).try_into()?,
            radius: self.radius.parse()?,
        })
    }
}

impl TryInto<PolygonSetInputKind> for &InputState {
    type Error = String;

    fn try_into(self) -> Result<PolygonSetInputKind, Self::Error> {
        match self {
            InputState::Rect(input) => input.try_into(),
            InputState::Polygon(input) => input.try_into(),
            InputState::Circle(input) => input.try_into(),
            InputState::Ellipse(input) => input.try_into(),
            InputState::Cube(input) => input.try_into(),
            InputState::Cuboid(input) => input.try_into(),
            InputState::Cone(input) => input.try_into(),
            InputState::Cylinder(input) => input.try_into(),
            InputState::Sphere(input) => input.try_into(),
        }
    }
}

#[derive(Debug)]
pub struct State {
    title: String,
    input_state: InputState,
}

impl From<Shape> for State {
    fn from(shape: Shape) -> Self {
        Self {
            title: shape.to_string(),
            input_state: match shape {
                Shape::Rectangle => InputState::Rect(RectInputState::default()),
                Shape::Polygon => InputState::Polygon(PolygonInputState::default()),
                Shape::Circle => InputState::Circle(CircleInputState::default()),
                Shape::Ellipse => InputState::Ellipse(EllipseInputState::default()),
                Shape::Cube => InputState::Cube(CubeInputState::default()),
                Shape::Cuboid => InputState::Cuboid(CuboidInputState::default()),
                Shape::Cone => InputState::Cone(ConeInputState::default()),
                Shape::Cylinder => InputState::Cylinder(CylinderInputState::default()),
                Shape::Sphere => InputState::Sphere(SphereInputState::default()),
            },
        }
    }
}

impl State {
    fn dialog_title(&self) -> &str {
        &self.title
    }
}

pub enum Response {
    Noop,
    Input(Result<PolygonSetInputKind, String>),
    Cancel,
}

pub fn show_in_ui(state: &mut State, ui: &mut Ui) -> Response {
    let mut response = Response::Noop;
    ui.group(|ui| {
        ui.label(egui::RichText::new(state.dialog_title()).strong());
        ui.add_space(INPUT_SECTION_MARGIN);
        input_table_layout(&mut state.input_state, ui);
        ui.add_space(INPUT_SECTION_MARGIN);
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            if ui.button("Cancel").clicked() {
                response = Response::Cancel;
            }
            if ui.button("Apply").clicked() {
                response = Response::Input((&state.input_state).try_into());
            }
        });
    });
    response
}

fn input_table_layout(input_state: &mut InputState, ui: &mut Ui) {
    match input_state {
        InputState::Rect(input) => rectangle_dialog_body(ui, input),
        InputState::Polygon(input) => polygon_dialog_body(ui, input),
        InputState::Circle(input) => circle_dialog_body(ui, input),
        InputState::Ellipse(input) => ellipse_dialog_body(ui, input),
        InputState::Cube(input) => cube_dialog_body(ui, input),
        InputState::Cuboid(input) => cuboid_dialog_body(ui, input),
        InputState::Cone(input) => cone_dialog_body(ui, input),
        InputState::Cylinder(input) => cylinder_dialog_body(ui, input),
        InputState::Sphere(input) => sphere_dialog_body(ui, input),
    }
}

fn x_y_input_field(ui: &mut Ui, x: &mut String, y: &mut String) {
    use dialog_utils::Field;
    dialog_utils::single_line_double_input_field(
        ui,
        Field {
            name: "X",
            value: x,
        },
        Field {
            name: "Y",
            value: y,
        },
    );
}

fn x_y_z_input_field(ui: &mut Ui, x: &mut String, y: &mut String, z: &mut String) {
    ui.horizontal(|ui| {
        ui.label("X");
        ui.text_edit_singleline(x);
        ui.label("Y");
        ui.text_edit_singleline(y);
        ui.label("Z");
        ui.text_edit_singleline(z);
    });
}

fn rectangle_dialog_body(ui: &mut Ui, input: &mut RectInputState) {
    ui.horizontal(|ui| {
        ui.label("Top left");
        x_y_input_field(ui, &mut input.top_left.x, &mut input.top_left.y);
    });
    ui.horizontal(|ui| {
        ui.label("Bottom right");
        x_y_input_field(ui, &mut input.bottom_right.x, &mut input.bottom_right.y);
    });
}

fn polygon_dialog_body(ui: &mut Ui, input: &mut PolygonInputState) {
    if let Some(err) = &input.err {
        if error_dialog::show(err, ui.ctx()).closed() {
            input.err = None;
        }
    }
    if let Some(receiver) = input.output_receiver.take() {
        match receiver.try_recv() {
            Ok(result) => match result {
                Ok(points) => {
                    if points.len() < 3 {
                        input.err = Some(String::from("File does not contain at least 3 points"));
                    } else {
                        input.vertices.points =
                            points.iter().map(RationalPointInput::from).collect();
                    }
                }
                Err(err) => input.err = Some(err),
            },
            Err(err) => match err {
                TryRecvError::Empty => {
                    input.output_receiver = Some(receiver);
                    ui.horizontal(|ui| {
                        ui.label("Parsing file...");
                        ui.spinner();
                    });
                    return;
                }
                TryRecvError::Disconnected => {
                    panic!("Parser worker thread crashed")
                }
            },
        }
    }
    ScrollArea::vertical().show(ui, |ui| {
        input
            .vertices
            .iter_mut()
            .enumerate()
            .for_each(|(index, point_input)| {
                ui.horizontal(|ui| {
                    ui.label(format!("Point {}", index + 1));
                    x_y_input_field(ui, &mut point_input.x, &mut point_input.y);
                });
            });
    });
    ui.horizontal(|ui| {
        ui.group(|ui| {
            if ui.button("Add row").clicked() {
                input.vertices.push_default();
            }
            let response = ui.add_enabled(input.vertices.can_pop(), Button::new("Remove row"));
            if response.clicked() {
                input.vertices.pop();
            }
        });
    });
    let path_opt = ui
        .button("Read from file")
        .clicked()
        .then(|| FileDialog::new().add_filter("csv", &["csv"]).pick_file())
        .flatten();
    let Some(path) = path_opt else {
        return;
    };
    let (output_sender, output_receiver) = mpsc::channel();
    input.parser().parse(PVData::new(path, output_sender));
    input.output_receiver = Some(output_receiver);
}

fn circle_dialog_body(ui: &mut Ui, input: &mut CircleInputState) {
    ui.horizontal(|ui| {
        ui.label("Origin");
        x_y_input_field(ui, &mut input.center.x, &mut input.center.y);
    });
    ui.horizontal(|ui| {
        ui.label("Diameter");
        ui.text_edit_singleline(&mut input.diameter);
    });
}

fn ellipse_dialog_body(ui: &mut Ui, input: &mut EllipseInputState) {
    ui.horizontal(|ui| {
        ui.label("Origin");
        x_y_input_field(ui, &mut input.center.x, &mut input.center.y);
    });
    ui.horizontal(|ui| {
        ui.label("Width");
        ui.text_edit_singleline(&mut input.width);
    });
    ui.horizontal(|ui| {
        ui.label("Height");
        ui.text_edit_singleline(&mut input.height);
    });
}

fn cube_dialog_body(ui: &mut Ui, input: &mut CubeInputState) {
    ui.horizontal(|ui| {
        ui.label("Origin");
        x_y_z_input_field(ui, &mut input.center.x, &mut input.center.y, &mut input.center.z);
    });
    ui.horizontal(|ui| {
        ui.label("Side Length");
        ui.text_edit_singleline(&mut input.side_length);
    });
}

fn cuboid_dialog_body(ui: &mut Ui, input: &mut CuboidInputState) {
    ui.horizontal(|ui| {
        ui.label("Origin");
        x_y_z_input_field(ui, &mut input.center.x, &mut input.center.y, &mut input.center.z);
    });
    ui.horizontal(|ui| {
        ui.label("Width");
        ui.text_edit_singleline(&mut input.width);
    });
    ui.horizontal(|ui| {
        ui.label("Height");
        ui.text_edit_singleline(&mut input.height);
    });
    ui.horizontal(|ui| {
        ui.label("Depth");
        ui.text_edit_singleline(&mut input.depth);
    });
}

fn cone_dialog_body(ui: &mut Ui, input: &mut ConeInputState) {
    ui.horizontal(|ui| {
        ui.label("Origin");
        x_y_z_input_field(ui, &mut input.center.x, &mut input.center.y, &mut input.center.z);
    });
    ui.horizontal(|ui| {
        ui.label("Radius");
        ui.text_edit_singleline(&mut input.radius);
    });
    ui.horizontal(|ui| {
        ui.label("Height");
        ui.text_edit_singleline(&mut input.height);
    });
}

fn cylinder_dialog_body(ui: &mut Ui, input: &mut CylinderInputState) {
    ui.horizontal(|ui| {
        ui.label("Origin");
        x_y_z_input_field(ui, &mut input.center.x, &mut input.center.y, &mut input.center.z);
    });
    ui.horizontal(|ui| {
        ui.label("Radius");
        ui.text_edit_singleline(&mut input.radius);
    });
    ui.horizontal(|ui| {
        ui.label("Height");
        ui.text_edit_singleline(&mut input.height);
    });
}

fn sphere_dialog_body(ui: &mut Ui, input: &mut SphereInputState) {
    ui.horizontal(|ui| {
        ui.label("Origin");
        x_y_z_input_field(ui, &mut input.center.x, &mut input.center.y, &mut input.center.z);
    });
    ui.horizontal(|ui| {
        ui.label("Radius");
        ui.text_edit_singleline(&mut input.radius);
    });
}
