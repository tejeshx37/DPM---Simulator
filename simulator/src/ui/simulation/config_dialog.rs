use crate::{
    model::engine::{self, Config, ExportConfig},
    ui::{
        always_open_window::AlwaysOpenWindow,
        dialog_utils::{self, ok_cancel},
    },
};
use cpd::{
    BulkMaterialProps, ElasticityCondition, FailureCriteria, IsotropicMaterialProps, MaterialProps,
    OrthotropicMaterialProps,
};
use egui::{Align, Checkbox, ComboBox, Context, Layout, Ui};
use enum_map::{Enum, EnumMap};
use mesh::Mesh;
use nalgebra_ext::matrix3::Component;
use rfd::FileDialog;
use std::{path::PathBuf, sync::Arc, time::Duration};
use strum::{AsRefStr, Display, EnumIter, IntoEnumIterator};

const INPUT_SECTION_MARGIN: f32 = 8.0;

// ─── Material Presets ────────────────────────────────────────────────────────

struct MaterialPreset {
    name: &'static str,
    elasticity_modulus: f32,  // Pa
    poissons_ratio: f32,
    density: f32,             // kg/m³
    damping: f32,
}

const MATERIAL_PRESETS: &[MaterialPreset] = &[
    MaterialPreset { name: "Steel (A36)",        elasticity_modulus: 200e9, poissons_ratio: 0.26, density: 7850.0, damping: 0.05 },
    MaterialPreset { name: "Aluminum (6061)",    elasticity_modulus: 69e9,  poissons_ratio: 0.33, density: 2700.0, damping: 0.02 },
    MaterialPreset { name: "Concrete (C30)",     elasticity_modulus: 30e9,  poissons_ratio: 0.20, density: 2400.0, damping: 0.10 },
    MaterialPreset { name: "Titanium (Ti-6Al)",  elasticity_modulus: 114e9, poissons_ratio: 0.34, density: 4430.0, damping: 0.03 },
    MaterialPreset { name: "Copper (pure)",      elasticity_modulus: 110e9, poissons_ratio: 0.35, density: 8960.0, damping: 0.04 },
    MaterialPreset { name: "HDPE Plastic",       elasticity_modulus: 1.1e9, poissons_ratio: 0.44, density: 960.0,  damping: 0.08 },
    MaterialPreset { name: "Glass (soda-lime)",  elasticity_modulus: 70e9,  poissons_ratio: 0.23, density: 2500.0, damping: 0.02 },
    MaterialPreset { name: "Rubber (natural)",   elasticity_modulus: 0.01e9,poissons_ratio: 0.49, density: 1100.0, damping: 0.20 },
];

// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Enum, Display, AsRefStr, EnumIter)]
#[strum(serialize_all = "title_case")]
enum SimulationConfigInput {
    Duration,
    RefreshPeriod,
    BodyForceX,
    BodyForceY,
    BodyForceZ,
}

impl SimulationConfigInput {
    fn info_text(&self) -> &str {
        match self {
            SimulationConfigInput::Duration => "Total duration of the simulation.",
            SimulationConfigInput::RefreshPeriod => "Rate at which new frames should be displayed. For example \
            a refresh rate of 100 will display new frame at every 100 timesteps. Higher the period, slower the update, \
            but faster the simulation since rendering a frame takes up a lot of time.",
            SimulationConfigInput::BodyForceX => "Body force component in the X direction (e.g. acceleration like gravity).",
            SimulationConfigInput::BodyForceY => "Body force component in the Y direction (e.g. acceleration like gravity).",
            SimulationConfigInput::BodyForceZ => "Body force component in the Z direction (e.g. acceleration like gravity).",
        }
    }
}

#[derive(Debug, Clone, Copy, Enum, Display, AsRefStr, EnumIter)]
#[strum(serialize_all = "title_case")]
enum BulkMaterialPropsInput {
    Density,
    Damping,
    FailureStrainEnergy,
    FailureTensionalStress,
    FailureCompressionalStress,
}

impl BulkMaterialPropsInput {
    fn info_text(&self) -> &str {
        match self {
            BulkMaterialPropsInput::Density => "Density of the material",
            BulkMaterialPropsInput::Damping => "Damping constant of the material",
            BulkMaterialPropsInput::FailureStrainEnergy => {
                "Strain energy at which cracks start to form.\nOptional parameter, leave blank if not necessary."
            }
            BulkMaterialPropsInput::FailureTensionalStress => {
                "Tensional stress at which cracks start to form.\nOptional parameter, leave blank if not necessary."
            }
            BulkMaterialPropsInput::FailureCompressionalStress => {
                "Compressional stress at which cracks starts to form.\nOptional parameter, leave blank if not necessary."
            }
        }
    }
}

#[derive(Debug, Default)]
struct IsotropicMaterialPropsInput {
    elasticity_modulus: String,
    poissons_ratio: String,
    elasticity_condition: ElasticityCondition,
}

impl From<&IsotropicMaterialProps> for IsotropicMaterialPropsInput {
    fn from(value: &IsotropicMaterialProps) -> Self {
        Self {
            elasticity_modulus: value.elasticity_modulus().to_string(),
            poissons_ratio: value.poissons_ratio().to_string(),
            elasticity_condition: *value.elasticity_condition(),
        }
    }
}

#[derive(Debug, Default)]
struct OrthotropicMaterialPropsInput {
    elasticity_modulus_x: String,
    elasticity_modulus_y: String,
    elasticity_modulus_z: String,
    poissons_ratio_xy: String,
    poissons_ratio_yx: String,
    poissons_ratio_yz: String,
    poissons_ratio_zy: String,
    poissons_ratio_zx: String,
    poissons_ratio_xz: String,
    shear_modulus_xy: String,
    shear_modulus_yz: String,
    shear_modulus_zx: String,
}

impl From<&OrthotropicMaterialProps> for OrthotropicMaterialPropsInput {
    fn from(value: &OrthotropicMaterialProps) -> Self {
        Self {
            elasticity_modulus_x: value.elasticity_modulus_x().to_string(),
            elasticity_modulus_y: value.elasticity_modulus_y().to_string(),
            elasticity_modulus_z: value.elasticity_modulus_z().to_string(),
            poissons_ratio_xy: value.poissons_ratio_xy().to_string(),
            poissons_ratio_yx: value.poissons_ratio_yx().to_string(),
            poissons_ratio_yz: value.poissons_ratio_yz().to_string(),
            poissons_ratio_zy: value.poissons_ratio_zy().to_string(),
            poissons_ratio_zx: value.poissons_ratio_zx().to_string(),
            poissons_ratio_xz: value.poissons_ratio_xz().to_string(),
            shear_modulus_xy: value.shear_modulus_xy().to_string(),
            shear_modulus_yz: value.shear_modulus_yz().to_string(),
            shear_modulus_zx: value.shear_modulus_zx().to_string(),
        }
    }
}

#[derive(Debug)]
enum MaterialPropsInput {
    Isotropic(IsotropicMaterialPropsInput),
    Orthotropic(OrthotropicMaterialPropsInput),
}

impl Default for MaterialPropsInput {
    fn default() -> Self {
        Self::Isotropic(IsotropicMaterialPropsInput::default())
    }
}

impl From<&MaterialProps> for MaterialPropsInput {
    fn from(value: &MaterialProps) -> Self {
        match value {
            MaterialProps::Isotropic(value) => {
                MaterialPropsInput::Isotropic(IsotropicMaterialPropsInput::from(value))
            }
            MaterialProps::Orthotropic(value) => {
                MaterialPropsInput::Orthotropic(OrthotropicMaterialPropsInput::from(value))
            }
        }
    }
}

#[derive(Debug, Default)]
struct ExportConfigInput {
    export_points: bool,
    exported_stress_components: EnumMap<Component, bool>,
    export_period: String,
    export_path: PathBuf,
    export_format: engine::ExportFormat,
}

#[derive(Debug)]
pub struct State {
    simulation_config_input: EnumMap<SimulationConfigInput, String>,
    time_step_input: String,
    adaptive_time_step: bool,
    min_time_delta_input: String,
    max_time_delta_input: String,
    bulk_material_props_input: EnumMap<BulkMaterialPropsInput, String>,
    material_props_input: MaterialPropsInput,
    export: bool,
    export_config_input: ExportConfigInput,
    mesh: Arc<Mesh>,
    use_gpu: bool,
    gpu_available: bool,
}

impl State {
    pub fn default(mesh: Arc<Mesh>) -> Self {
        Self {
            simulation_config_input: EnumMap::default(),
            time_step_input: String::default(),
            adaptive_time_step: true,
            min_time_delta_input: String::from("1e-9"),
            max_time_delta_input: String::from("1e-2"),
            bulk_material_props_input: EnumMap::default(),
            material_props_input: MaterialPropsInput::default(),
            export: false,
            export_config_input: ExportConfigInput::default(),
            mesh,
            use_gpu: false,
            gpu_available: false,
        }
    }

    pub fn new(config: &Config, mesh: Arc<Mesh>) -> Self {
        let mp = config.cpd_config().material_props();
        let bp = mp.bulk_props();
        let fc = bp.failure_criteria();
        let ec = config.export_config();
        let opt_to_string =
            |opt: &Option<f32>| opt.map(|value| value.to_string()).unwrap_or_default();
        Self {
            simulation_config_input: enum_map::enum_map! {
                SimulationConfigInput::Duration => config.cpd_config().duration().as_secs_f32().to_string(),
                SimulationConfigInput::RefreshPeriod => config.refresh_period().to_string(),
                SimulationConfigInput::BodyForceX => config.cpd_config().body_force().x.to_string(),
                SimulationConfigInput::BodyForceY => config.cpd_config().body_force().y.to_string(),
                SimulationConfigInput::BodyForceZ => config.cpd_config().body_force().z.to_string(),
            },
            time_step_input: format!("{:e}", config.cpd_config().time_delta().as_secs_f64()),
            adaptive_time_step: *config.cpd_config().adaptive_time_step(),
            min_time_delta_input: config.cpd_config().min_time_delta().map(|d| format!("{:e}", d.as_secs_f64())).unwrap_or_else(|| String::from("1e-9")),
            max_time_delta_input: config.cpd_config().max_time_delta().map(|d| format!("{:e}", d.as_secs_f64())).unwrap_or_else(|| String::from("1e-2")),
            bulk_material_props_input: enum_map::enum_map! {
                BulkMaterialPropsInput::Density => bp.density().to_string(),
                BulkMaterialPropsInput::Damping => bp.damping().to_string(),
                BulkMaterialPropsInput::FailureStrainEnergy => opt_to_string(fc.strain_energy()) ,
                BulkMaterialPropsInput::FailureTensionalStress => opt_to_string(fc.tensional_stress()),
                BulkMaterialPropsInput::FailureCompressionalStress => opt_to_string(fc.compressional_stress()),
            },
            material_props_input: MaterialPropsInput::from(mp),
            export: ec.is_some(),
            export_config_input: ec
                .as_ref()
                .map(|ec| ExportConfigInput {
                    export_points: *ec.export_points(),
                    exported_stress_components: *ec.export_stress_components(),
                    export_period: ec.export_period().to_string(),
                    export_path: ec.export_path().to_owned(),
                    export_format: *ec.export_format(),
                })
                .unwrap_or_default(),
            mesh,
            use_gpu: *config.use_gpu(),
            gpu_available: false, // Will be set by simulation page
        }
    }
    
    pub fn set_gpu_available(&mut self, available: bool) {
        self.gpu_available = available;
    }
}

impl TryFrom<&State> for Config {
    type Error = String;

    fn try_from(state: &State) -> Result<Self, Self::Error> {
        macro_rules! parse_f32 {
            ($name:expr, $value:expr) => {
                $value
                    .parse()
                    .map_err(|_| format!("Invalid {} {}", $name, $value))
                    .and_then(|value: f32| {
                        if !value.is_finite() {
                            Err(format!("{} should be finite, i.e. no NaN or Inf", $name))
                        } else {
                            Ok(value)
                        }
                    })
            };
        }
        macro_rules! parse_input {
            ($inputs_field:ident, $variant:ident) => {{
                let input = paste::paste! { [< $inputs_field:camel >]::$variant };
                let value = &state.$inputs_field[input];
                parse_f32!(input, value)
            }};
        }
        macro_rules! failure_criteria {
            ( $( $criteria:ident ),* ) => {{
                let builder = FailureCriteria::builder();
                $(
                    let builder = {
                        let variant = paste::paste! { BulkMaterialPropsInput::[< Failure $criteria >] };
                        let input = &state.bulk_material_props_input[variant];
                        let value = (!input.is_empty()).then(|| input
                            .parse()
                            .map_err(|_| format!("Invalid {variant} {input}")))
                            .transpose()?;
                        paste::paste! { builder.[< $criteria:snake >](value) }
                    };
                )*
                builder.build()
            }};
        }
        let failure_critera = failure_criteria!(StrainEnergy, TensionalStress, CompressionalStress);
        let ec = &state.export_config_input;
        let bulk_props = BulkMaterialProps::builder()
            .density(parse_input!(bulk_material_props_input, Density)?)
            .damping(parse_input!(bulk_material_props_input, Damping)?)
            .failure_criteria(failure_critera)
            .build();
        let material_props = match &state.material_props_input {
            MaterialPropsInput::Isotropic(input) => MaterialProps::Isotropic(
                IsotropicMaterialProps::builder()
                    .bulk_props(bulk_props)
                    .elasticity_condition(input.elasticity_condition)
                    .elasticity_modulus(parse_f32!("elasticity modulus", input.elasticity_modulus)?)
                    .poissons_ratio(parse_f32!("poisson's ratio", input.poissons_ratio)?)
                    .build(),
            ),
            MaterialPropsInput::Orthotropic(input) => {
                let ex = parse_f32!("Ex", input.elasticity_modulus_x)?;
                let ey = parse_f32!("Ey", input.elasticity_modulus_y)?;
                let ez = parse_f32!("Ez", input.elasticity_modulus_z)?;
                let vxy = parse_f32!("Vxy", input.poissons_ratio_xy)?;
                let vyx = parse_f32!("Vyx", input.poissons_ratio_yx)?;
                let vyz = parse_f32!("Vyz", input.poissons_ratio_yz)?;
                let vzy = parse_f32!("Vzy", input.poissons_ratio_zy)?;
                let vzx = parse_f32!("Vzx", input.poissons_ratio_zx)?;
                let vxz = parse_f32!("Vxz", input.poissons_ratio_xz)?;
                let gxy = parse_f32!("Gxy", input.shear_modulus_xy)?;
                let gyz = parse_f32!("Gyz", input.shear_modulus_yz)?;
                let gzx = parse_f32!("Gzx", input.shear_modulus_zx)?;
                let ortho = OrthotropicMaterialProps::builder()
                    .bulk_props(bulk_props)
                    .elasticity_modulus_x(ex)
                    .elasticity_modulus_y(ey)
                    .elasticity_modulus_z(ez)
                    .poissons_ratio_xy(vxy)
                    .poissons_ratio_yx(vyx)
                    .poissons_ratio_yz(vyz)
                    .poissons_ratio_zy(vzy)
                    .poissons_ratio_zx(vzx)
                    .poissons_ratio_xz(vxz)
                    .shear_modulus_xy(gxy)
                    .shear_modulus_yz(gyz)
                    .shear_modulus_zx(gzx)
                    .build();
                ortho.validate()?;
                MaterialProps::Orthotropic(ortho)
            }
        };
        let min_dt = if state.adaptive_time_step && !state.min_time_delta_input.is_empty() {
            Some(
                state.min_time_delta_input.parse()
                    .map(Duration::from_secs_f32)
                    .map_err(|_| format!("Invalid min time step {}", state.min_time_delta_input))?
            )
        } else {
            None
        };
        
        let max_dt = if state.adaptive_time_step && !state.max_time_delta_input.is_empty() {
            Some(
                state.max_time_delta_input.parse()
                    .map(Duration::from_secs_f32)
                    .map_err(|_| format!("Invalid max time step {}", state.max_time_delta_input))?
            )
        } else {
            None
        };

        let cpd_config = cpd::config::Config::builder()
            .material_props(material_props)
            .duration(parse_input!(simulation_config_input, Duration).map(Duration::from_secs_f32)?)
            .time_delta(
                state
                    .time_step_input
                    .parse()
                    .map(Duration::from_secs_f32)
                    .map_err(|_| format!("Invalid time step {}", state.time_step_input))?,
            )
            .adaptive_time_step(state.adaptive_time_step)
            .min_time_delta(min_dt)
            .max_time_delta(max_dt)
            .body_force(nalgebra::Vector3::new(
                parse_input!(simulation_config_input, BodyForceX)?,
                parse_input!(simulation_config_input, BodyForceY)?,
                parse_input!(simulation_config_input, BodyForceZ)?,
            ))
            .build();
        let export_config =
            state
                .export
                .then(|| {
                    if !ec.export_path.is_dir() {
                        Err(format!(
                            "Export path '{}' does not exist",
                            ec.export_path.display()
                        ))
                    } else {
                        Ok(ExportConfig::builder()
                            .export_points(ec.export_points)
                            .export_stress_components(ec.exported_stress_components)
                            .export_period(ec.export_period.parse().map_err(|_| {
                                format!("Invalid export period {}", ec.export_period)
                            })?)
                            .export_path(ec.export_path.to_owned())
                            .export_format(ec.export_format)
                            .build())
                    }
                })
                .transpose()?;
        let config = Self::builder()
            .cpd_config(cpd_config)
            .refresh_period({
                let value = &state.simulation_config_input[SimulationConfigInput::RefreshPeriod];
                value
                    .parse()
                    .map_err(|_| format!("Invalid refresh period {value}"))?
            })
            .export_config(export_config)
            .use_gpu(state.use_gpu)
            .build();
        Ok(config)
    }
}

pub enum Response {
    Noop,
    ConfigResult(Result<Box<Config>, String>),
    Cancel,
}

pub fn show(state: &mut State, ctx: &Context) -> Response {
    AlwaysOpenWindow::new("Engine config")
        .resizable(false)
        .show(ctx, |ui| window_ui(state, ui))
}

fn window_ui(state: &mut State, ui: &mut Ui) -> Response {
    ui.add_space(INPUT_SECTION_MARGIN);
    ui.group(|ui| simulation_config_table_layout(state, ui));
    ui.add_space(INPUT_SECTION_MARGIN);
    // --- Material Presets Dropdown ---
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.strong("Quick Material Preset:");
            ComboBox::from_id_salt("material_preset_combo")
                .selected_text("Select a preset...")
                .show_ui(ui, |ui| {
                    for preset in MATERIAL_PRESETS {
                        if ui.selectable_label(false, preset.name).clicked() {
                            // Apply preset values to isotropic input
                            state.material_props_input = MaterialPropsInput::Isotropic(
                                IsotropicMaterialPropsInput {
                                    elasticity_modulus: preset.elasticity_modulus.to_string(),
                                    poissons_ratio: preset.poissons_ratio.to_string(),
                                    elasticity_condition: ElasticityCondition::ThreeDimensional,
                                },
                            );
                            state.bulk_material_props_input[BulkMaterialPropsInput::Density] =
                                preset.density.to_string();
                            state.bulk_material_props_input[BulkMaterialPropsInput::Damping] =
                                preset.damping.to_string();
                        }
                    }
                });
        });
    });
    ui.add_space(INPUT_SECTION_MARGIN);
    ui.group(|ui| material_props_table_layout(state, ui));
    ui.add_space(INPUT_SECTION_MARGIN);
    ui.group(|ui| time_step_input(state, ui));
    ui.add_space(INPUT_SECTION_MARGIN);
    ui.horizontal(|ui| {
        ui.checkbox(&mut state.use_gpu, "Enable GPU Acceleration")
            .on_hover_text("Use hardware acceleration for physics calculations. Recommended for large meshes.");
        if !state.gpu_available {
            ui.label(egui::RichText::new("(Hardware not available)").small().weak().color(egui::Color32::GRAY));
        }
    });
    ui.add_space(INPUT_SECTION_MARGIN);
    ui.checkbox(&mut state.export, "Export data")
        .on_hover_text("Check to configure which all data should be exported.");
    if state.export {
        ui.add_space(INPUT_SECTION_MARGIN);
        ui.group(|ui| export_config_layout(&mut state.export_config_input, ui));
    }
    ui.add_space(INPUT_SECTION_MARGIN);
    ui.with_layout(
        Layout::right_to_left(Align::Min),
        |ui| match ok_cancel::buttons(ui) {
            ok_cancel::Response::Ok => {
                Response::ConfigResult(Config::try_from(&*state).map(Box::new))
            }
            ok_cancel::Response::Cancel => Response::Cancel,
            ok_cancel::Response::Noop => Response::Noop,
        },
    )
    .inner
}

fn time_step_input(state: &mut State, ui: &mut Ui) {
    if let Some(optimal_time_step) = optimal_time_step(state) {
        ui.horizontal(|ui| {
            ui.label(format!("Optimal time step is {optimal_time_step:e}"));
            if ui.button("Use this value").clicked() {
                state.time_step_input = format!("{:e}", optimal_time_step);
            }
        });
    }
    ui.horizontal(|ui| {
        ui.label("Initial Time step");
        ui.text_edit_singleline(&mut state.time_step_input)
            .on_hover_text(
                "Time increment (dt) at each iteration of the simulation.\n\
        Smaller the time step, slower and better the simulation.\n\
        Higher the time step, faster but very inaccurate simulation.",
            );
    });
    
    ui.checkbox(&mut state.adaptive_time_step, "Enable Adaptive Time Step (CFL)")
        .on_hover_text("Automatically adjusts dt based on Courant number to prevent explosions and speed up stable simulation.");
        
    if state.adaptive_time_step {
        ui.indent("cfl_indent", |ui| {
            ui.horizontal(|ui| {
                ui.label("Min dt");
                ui.text_edit_singleline(&mut state.min_time_delta_input);
                ui.label("Max dt");
                ui.text_edit_singleline(&mut state.max_time_delta_input);
            });
        });
    }
}

fn optimal_time_step(state: &State) -> Option<f64> {
    state.bulk_material_props_input[BulkMaterialPropsInput::Density]
        .parse::<f64>()
        .ok()
        .and_then(|density| {
            let e_opt = match &state.material_props_input {
                MaterialPropsInput::Isotropic(p) => p.elasticity_modulus.parse::<f64>().ok(),
                MaterialPropsInput::Orthotropic(p) => p
                    .elasticity_modulus_x
                    .parse::<f64>()
                    .ok()
                    .zip(p.elasticity_modulus_y.parse().ok())
                    .map(|(ex, ey)| ex.max(ey)),
            };
            e_opt.map(|e| (density, e))
        })
        .and_then(|(density, elasticity_modulus)| {
            engine::optimal_time_delta(density, elasticity_modulus, &state.mesh)
        })
}

fn simulation_config_table_layout(state: &mut State, ui: &mut Ui) {
    SimulationConfigInput::iter().for_each(|input| {
        num_input_row(
            ui,
            input.as_ref(),
            &mut state.simulation_config_input[input],
            input.info_text(),
        );
    });
}

fn bulk_material_props_input_layout(state: &mut State, ui: &mut Ui) {
    BulkMaterialPropsInput::iter().for_each(|input| {
        num_input_row(
            ui,
            input.as_ref(),
            &mut state.bulk_material_props_input[input],
            input.info_text(),
        );
    });
}

fn isotropic_material_props_input_layout(input: &mut IsotropicMaterialPropsInput, ui: &mut Ui) {
    num_input_row(
        ui,
        "Elasticity modulus",
        &mut input.elasticity_modulus,
        "Elasticity modulus of the material",
    );
    num_input_row(
        ui,
        "Poisson's ratio",
        &mut input.poissons_ratio,
        "Poisson's ratio of the material",
    );
    ui.horizontal(|ui| {
        ui.label("Elasticity condition:");
        ui.selectable_value(
            &mut input.elasticity_condition,
            ElasticityCondition::ThreeDimensional,
            "Three Dimensional",
        );
    });
}

fn orthotropic_material_props_input_layout(input: &mut OrthotropicMaterialPropsInput, ui: &mut Ui) {
    use dialog_utils::Field;
    ui.horizontal(|ui| {
        ui.label("Elasticity moduli (Ex, Ey, Ez):");
        dialog_utils::single_line_double_input_field(
            ui,
            Field { name: "Ex", value: &mut input.elasticity_modulus_x },
            Field { name: "Ey", value: &mut input.elasticity_modulus_y },
        );
        ui.label("Ez:");
        ui.add(egui::TextEdit::singleline(&mut input.elasticity_modulus_z).desired_width(60.0));
    });
    ui.horizontal(|ui| {
        ui.label("Poisson's ratios (XY, YX):");
        dialog_utils::single_line_double_input_field(
            ui,
            Field { name: "Vxy", value: &mut input.poissons_ratio_xy },
            Field { name: "Vyx", value: &mut input.poissons_ratio_yx },
        );
    });
    ui.horizontal(|ui| {
        ui.label("Poisson's ratios (YZ, ZY):");
        dialog_utils::single_line_double_input_field(
            ui,
            Field { name: "Vyz", value: &mut input.poissons_ratio_yz },
            Field { name: "Vzy", value: &mut input.poissons_ratio_zy },
        );
    });
    ui.horizontal(|ui| {
        ui.label("Poisson's ratios (ZX, XZ):");
        dialog_utils::single_line_double_input_field(
            ui,
            Field { name: "Vzx", value: &mut input.poissons_ratio_zx },
            Field { name: "Vxz", value: &mut input.poissons_ratio_xz },
        );
    });
    ui.horizontal(|ui| {
        ui.label("Shear moduli (Gxy, Gyz, Gzx):");
        ui.add(egui::TextEdit::singleline(&mut input.shear_modulus_xy).desired_width(60.0));
        ui.add(egui::TextEdit::singleline(&mut input.shear_modulus_yz).desired_width(60.0));
        ui.add(egui::TextEdit::singleline(&mut input.shear_modulus_zx).desired_width(60.0));
    });
}

fn material_props_table_layout(state: &mut State, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label("Material kind:");
        let response = ui.selectable_label(
            matches!(state.material_props_input, MaterialPropsInput::Isotropic(_)),
            "Isotropic",
        );
        if response.clicked() {
            state.material_props_input =
                MaterialPropsInput::Isotropic(IsotropicMaterialPropsInput::default());
        }
        let response = ui.selectable_label(
            matches!(
                state.material_props_input,
                MaterialPropsInput::Orthotropic(_)
            ),
            "Orthotropic",
        );
        if response.clicked() {
            state.material_props_input =
                MaterialPropsInput::Orthotropic(OrthotropicMaterialPropsInput::default());
        }
    });
    match &mut state.material_props_input {
        MaterialPropsInput::Isotropic(input) => {
            isotropic_material_props_input_layout(input, ui);
        }
        MaterialPropsInput::Orthotropic(input) => {
            orthotropic_material_props_input_layout(input, ui);
        }
    }
    bulk_material_props_input_layout(state, ui);
}

fn export_config_layout(state: &mut ExportConfigInput, ui: &mut Ui) {
    ui.with_layout(ui.layout().with_cross_align(Align::Min), |ui| {
        ui.horizontal(|ui| {
            ui.label("Export points");
            ui.add(Checkbox::without_text(&mut state.export_points))
                .on_hover_text("Check to export points to a file named Points_<timestep>.csv");
        });
        ui.horizontal(|ui| {
            ui.label("Export stress components");
            Component::iter().for_each(|comp| {
                ui.checkbox(&mut state.exported_stress_components[comp], comp.as_ref())
                    .on_hover_text("Check to export this component of stress to a file name Stress_<timestep>.csv");
            });
        });
        ui.horizontal(|ui| {
            ui.label("Export format:");
            engine::ExportFormat::iter().for_each(|format| {
                ui.selectable_value(&mut state.export_format, format, format.as_ref());
            });
        });
        ui.horizontal(|ui| {
            ui.label("Export period");
            ui.text_edit_singleline(&mut state.export_period).on_hover_text("Time step interval at \
            which data should be exported.\nFor example, for an export period of 100, at every 100th \
            time step, data will be exported.");
        });
        ui.horizontal(|ui| {
            let opt = ui
                .button("Select export path")
                .clicked()
                .then(|| {
                    FileDialog::new()
                        .set_directory(&state.export_path)
                        .pick_folder()
                })
                .flatten();
            if let Some(path) = opt {
                state.export_path = path;
            }
            ui.label(state.export_path.display().to_string()).on_hover_text("The path to which \
            files should be written to.\nCannot be empty");
        });
    });
}

fn num_input_row(ui: &mut Ui, label: &str, text: &mut String, info_text: &str) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.text_edit_singleline(text).on_hover_text(info_text);
    });
}
