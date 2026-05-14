use crate::ui::{always_open_window::AlwaysOpenWindow, dialog_utils::ok_cancel};
use egui::{Align, Context, Layout, Ui};
use mesh::{
    Region, SeedingConfig, SeedingPattern, SeedingRegion, SeedingStrategy,
};
use nalgebra::Vector3;

const INPUT_SECTION_MARGIN: f32 = 8.0;

#[derive(Debug)]
pub struct State {
    num_input: String,
    override_size_bound: bool,
    size_bound_input: String,
    thickness_input: String,
    /// Extra interior seed points for 3D polyhedron meshes (per-region strategies).
    pub regional_seeding_3d: bool,
    pub seeding_regions: Vec<SeedingRegion>,
    pub default_fill_3d: bool,
    pub default_pattern: SeedingPattern,
    pub default_density: f64,
    pub default_radius: f64,
}

impl Default for State {
    fn default() -> Self {
        Self {
            num_input: String::from("512"),
            override_size_bound: false,
            size_bound_input: String::from("0.5"),
            thickness_input: String::from("10.0"),
            regional_seeding_3d: false,
            seeding_regions: Vec::new(),
            default_fill_3d: false,
            default_pattern: SeedingPattern::Grid,
            default_density: 40.0,
            default_radius: 0.0,
        }
    }
}

pub struct Data {
    pub num_points: u32,
    pub size_bound_override: Option<f64>,
    pub thickness: f64,
    pub seeding_config: Option<SeedingConfig>,
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

        let seeding_config = if value.regional_seeding_3d {
            if value.seeding_regions.is_empty() && !value.default_fill_3d {
                return Err(String::from(
                    "3D regional seeding: add at least one region or enable “Fill remainder of solid”.",
                ));
            }
            let default_strategy = value.default_fill_3d.then(|| SeedingStrategy {
                pattern: value.default_pattern,
                density: value.default_density,
                radius: value.default_radius,
            });
            Some(SeedingConfig {
                regions: value.seeding_regions.clone(),
                default_strategy,
            })
        } else {
            None
        };

        Ok(Data {
            num_points,
            size_bound_override,
            thickness,
            seeding_config,
        })
    }
}

pub enum Response {
    Noop,
    DataResult(Result<Data, String>),
    Cancel,
}

pub fn show(
    state: &mut State,
    ctx: &Context,
    solid_aabb: Option<([f64; 3], [f64; 3])>,
) -> Response {
    AlwaysOpenWindow::new("Mesh config")
        .resizable(true)
        .default_width(420.0)
        .show(ctx, |ui| {
            instructions(ui);
            ui.checkbox(&mut state.override_size_bound, "Override size bound");
            ui.group(|ui| input_table_layout(state, ui));
            ui.add_space(INPUT_SECTION_MARGIN);
            regional_seeding_3d_ui(state, ui, solid_aabb);
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
        ui.add_space(6.0);
        ui.label(
            "For 3D solids, you can optionally add volumetric seed regions (axis-aligned box or sphere) \
            with independent pattern and density. Candidate points are clipped to lie inside the solid. \
            “Fill remainder” seeds the rest of the bounding box with another pattern.",
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

fn regional_seeding_3d_ui(
    state: &mut State,
    ui: &mut Ui,
    solid_aabb: Option<([f64; 3], [f64; 3])>,
) {
    let enabled = solid_aabb.is_some();
    ui.add_enabled_ui(enabled, |ui| {
        ui.collapsing("3D volumetric regional seeding", |ui| {
            if !enabled {
                ui.label("Available only for 3D polyhedron shapes.");
                return;
            }
            ui.checkbox(&mut state.regional_seeding_3d, "Enable regional interior seeding");
            if !state.regional_seeding_3d {
                return;
            }
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button("Add axis-aligned box").clicked() {
                    state
                        .seeding_regions
                        .push(default_box_region(solid_aabb));
                }
                if ui.button("Add sphere").clicked() {
                    state
                        .seeding_regions
                        .push(default_sphere_region(solid_aabb));
                }
            });
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Regions (each: geometry + pattern + density)")
                    .strong(),
            );
            ui.add_space(4.0);
            let mut remove: Option<usize> = None;
            for (i, sr) in state.seeding_regions.iter_mut().enumerate() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("Region {}", i + 1)).strong());
                        if ui.small_button("Remove").clicked() {
                            remove = Some(i);
                        }
                        if solid_aabb.is_some() && ui.small_button("Snap to solid AABB").clicked() {
                            if let Some((mn, mx)) = solid_aabb {
                                snap_region_to_aabb(&mut sr.region, mn, mx);
                            }
                        }
                    });
                    region_editor(ui, i, &mut sr.region, solid_aabb);
                    strategy_editor(ui, i, &mut sr.strategy);
                });
            }
            if let Some(i) = remove {
                state.seeding_regions.remove(i);
            }

            ui.add_space(10.0);
            ui.separator();
            ui.checkbox(
                &mut state.default_fill_3d,
                "Fill remainder of solid (outside listed regions)",
            );
            if state.default_fill_3d {
                ui.indent("def_fill", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Pattern");
                        pattern_combo(
                            ui,
                            egui::Id::new("mesh_def_fill_pat"),
                            &mut state.default_pattern,
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Density (pts / unit³)");
                        ui.add(
                            egui::DragValue::new(&mut state.default_density)
                                .speed(1.0)
                                .range(0.0..=1.0e6),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Radius");
                        ui.add(egui::DragValue::new(&mut state.default_radius).speed(0.01));
                    });
                });
            }
        });
    });
}

fn default_strategy() -> SeedingStrategy {
    SeedingStrategy {
        pattern: SeedingPattern::Grid,
        density: 40.0,
        radius: 0.0,
    }
}

fn default_box_region(solid_aabb: Option<([f64; 3], [f64; 3])>) -> SeedingRegion {
    let region = if let Some(([mnx, mny, mnz], [mxx, mxy, mxz])) = solid_aabb {
        Region::BoundingBox {
            min: Vector3::new(mnx, mny, mnz),
            max: Vector3::new(mxx, mxy, mxz),
        }
    } else {
        Region::BoundingBox {
            min: Vector3::new(-1.0, -1.0, -1.0),
            max: Vector3::new(1.0, 1.0, 1.0),
        }
    };
    SeedingRegion {
        region,
        strategy: default_strategy(),
    }
}

fn default_sphere_region(solid_aabb: Option<([f64; 3], [f64; 3])>) -> SeedingRegion {
    let (center, radius) = if let Some(([mnx, mny, mnz], [mxx, mxy, mxz])) = solid_aabb {
        let c = Vector3::new(
            0.5 * (mnx + mxx),
            0.5 * (mny + mxy),
            0.5 * (mnz + mxz),
        );
        let dx = mxx - mnx;
        let dy = mxy - mny;
        let dz = mxz - mnz;
        let r = 0.25 * (dx * dx + dy * dy + dz * dz).sqrt();
        (c, r.max(1e-6))
    } else {
        (Vector3::zeros(), 1.0)
    };
    SeedingRegion {
        region: Region::SDF { center, radius },
        strategy: SeedingStrategy {
            pattern: SeedingPattern::Fibonacci,
            density: 80.0,
            radius: 0.0,
        },
    }
}

fn snap_region_to_aabb(region: &mut Region, mn: [f64; 3], mx: [f64; 3]) {
    *region = Region::BoundingBox {
        min: Vector3::new(mn[0], mn[1], mn[2]),
        max: Vector3::new(mx[0], mx[1], mx[2]),
    };
}

fn pattern_combo(ui: &mut Ui, id: egui::Id, pattern: &mut SeedingPattern) {
    egui::ComboBox::new(id, "Pattern")
        .selected_text(pattern_label(*pattern))
        .show_ui(ui, |ui| {
            ui.selectable_value(pattern, SeedingPattern::Grid, "Grid");
            ui.selectable_value(pattern, SeedingPattern::Hexagonal, "Hexagonal");
            ui.selectable_value(pattern, SeedingPattern::Fibonacci, "Fibonacci");
            ui.selectable_value(pattern, SeedingPattern::Random, "Random");
        });
}

fn pattern_label(p: SeedingPattern) -> &'static str {
    match p {
        SeedingPattern::Grid => "Grid",
        SeedingPattern::Hexagonal => "Hexagonal",
        SeedingPattern::Fibonacci => "Fibonacci",
        SeedingPattern::Random => "Random",
    }
}

fn strategy_editor(ui: &mut Ui, row: usize, s: &mut SeedingStrategy) {
    ui.horizontal(|ui| {
        ui.label("Pattern");
        pattern_combo(ui, egui::Id::new(("seed_strat_pat", row)), &mut s.pattern);
    });
    ui.horizontal(|ui| {
        ui.label("Density (pts / unit³)");
        ui.add(
            egui::DragValue::new(&mut s.density)
                .speed(1.0)
                .range(0.0..=1.0e6),
        );
    });
    ui.horizontal(|ui| {
        ui.label("Radius (reserved)");
        ui.add(egui::DragValue::new(&mut s.radius).speed(0.01));
    });
}

fn region_editor(
    ui: &mut Ui,
    row: usize,
    region: &mut Region,
    solid_aabb: Option<([f64; 3], [f64; 3])>,
) {
    if matches!(region, Region::PolygonZone { .. }) {
        ui.label(
            egui::RichText::new(
                "Polygon prism region: edit via project data or replace with a box/sphere.",
            )
            .small()
            .weak(),
        );
        if ui.button("Replace with axis-aligned box").clicked() {
            if let Some((mn, mx)) = solid_aabb {
                snap_region_to_aabb(region, mn, mx);
            } else {
                *region = Region::BoundingBox {
                    min: Vector3::new(-1.0, -1.0, -1.0),
                    max: Vector3::new(1.0, 1.0, 1.0),
                };
            }
        }
        return;
    }

    let discr = match &region {
        Region::BoundingBox { .. } => 0,
        Region::SDF { .. } => 1,
        Region::PolygonZone { .. } => 0,
    };
    let mut k = discr;
    egui::ComboBox::new(egui::Id::new(("seed_reg_shape", row)), "Shape")
        .selected_text(match k {
            0 => "Axis-aligned box",
            1 => "Sphere (SDF)",
            _ => "Axis-aligned box",
        })
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut k, 0u8, "Axis-aligned box");
            ui.selectable_value(&mut k, 1u8, "Sphere (SDF)");
        });
    if k != discr {
        *region = match k {
            0 => Region::BoundingBox {
                min: Vector3::new(0.0, 0.0, 0.0),
                max: Vector3::new(1.0, 1.0, 1.0),
            },
            _ => Region::SDF {
                center: Vector3::zeros(),
                radius: 1.0,
            },
        };
        if let Some((mn, mx)) = solid_aabb {
            if matches!(region, Region::BoundingBox { .. }) {
                snap_region_to_aabb(region, mn, mx);
            }
        }
    }

    match region {
        Region::BoundingBox { min, max } => {
            ui.horizontal(|ui| {
                ui.label("min");
                ui.add(egui::DragValue::new(&mut min.x).speed(0.05));
                ui.add(egui::DragValue::new(&mut min.y).speed(0.05));
                ui.add(egui::DragValue::new(&mut min.z).speed(0.05));
            });
            ui.horizontal(|ui| {
                ui.label("max");
                ui.add(egui::DragValue::new(&mut max.x).speed(0.05));
                ui.add(egui::DragValue::new(&mut max.y).speed(0.05));
                ui.add(egui::DragValue::new(&mut max.z).speed(0.05));
            });
        }
        Region::SDF { center, radius } => {
            ui.horizontal(|ui| {
                ui.label("center");
                ui.add(egui::DragValue::new(&mut center.x).speed(0.05));
                ui.add(egui::DragValue::new(&mut center.y).speed(0.05));
                ui.add(egui::DragValue::new(&mut center.z).speed(0.05));
            });
            ui.horizontal(|ui| {
                ui.label("radius");
                ui.add(
                    egui::DragValue::new(radius)
                        .speed(0.05)
                        .range(1e-9..=1.0e9),
                );
            });
        }
        Region::PolygonZone { .. } => {}
    }
}
