use csv::WriterBuilder;
use derive_getters::Getters;
use enum_map::EnumMap;
use mesh::Mesh;
use nalgebra_ext::matrix3::Component;
use std::{
    fs, iter,
    path::{Path, PathBuf},
};
use strum::{AsRefStr, Display, EnumIter};
use typed_builder::TypedBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, Display, EnumIter, AsRefStr)]
pub enum ExportFormat {
    #[default]
    Csv,
    Pot,
}

#[derive(Debug, Clone, Getters, TypedBuilder)]
pub struct ExportConfig {
    export_points: bool,
    export_stress_components: EnumMap<Component, bool>,
    export_period: u128,
    export_path: PathBuf,
    #[builder(default)]
    export_format: ExportFormat,
}

pub fn mesh(mesh: &Mesh, path: &Path) -> csv::Result<()> {
    fs::create_dir_all(path)?;
    let faces = mesh.triangulation_data().faces();
    let mut writer = WriterBuilder::default()
        .buffer_capacity(faces.len())
        .from_path(path.join("Elements").with_extension("csv"))?;
    faces.iter().try_for_each(|face| writer.serialize(face.0))
}

pub fn data(data: &cpd::ExportData, config: &ExportConfig, time_step: u128) -> anyhow::Result<()> {
    fs::create_dir_all(&config.export_path)?;
    match config.export_format {
        ExportFormat::Csv => export_csv(data, config, time_step),
        ExportFormat::Pot => export_pot(data, config, time_step),
    }
}

fn export_pot(data: &cpd::ExportData, config: &ExportConfig, time_step: u128) -> anyhow::Result<()> {
    let path = config.export_path.join(format!("Data_{time_step}.pot"));
    let file = fs::File::create(path)?;
    pot::to_writer(data, file)?;
    Ok(())
}

fn export_csv(data: &cpd::ExportData, config: &ExportConfig, time_step: u128) -> anyhow::Result<()> {
    if config.export_points {
        let mut writer = WriterBuilder::default()
            .buffer_capacity(data.nodes().len())
            .has_headers(true)
            .from_path(
                config
                    .export_path
                    .join(format!("Points_{time_step}"))
                    .with_extension("csv"),
            )?;
        writer.write_record(["X", "Y", "Z"])?;
        data.nodes()
            .iter()
            .try_for_each(|node| {
                let p = node.position();
                writer.serialize((p.x, p.y, p.z))
            })?;
    }
    let header = config
        .export_stress_components
        .iter()
        .filter_map(|(component, export)| export.then_some(component))
        .map(|component| format!("E{}", component.as_ref()))
        .chain(iter::once(String::from("Broken")))
        .collect::<Vec<_>>();
    if header.len() <= 1 {
        return Ok(());
    }
    let mut writer = WriterBuilder::default()
        .buffer_capacity(data.elements().len())
        .has_headers(true)
        .from_path(
            config
                .export_path
                .join(format!("Stress_{time_step}"))
                .with_extension("csv"),
        )?;
    writer.write_record(&header)?;
    data.elements()
        .iter()
        .map(|element| {
            config
                .export_stress_components
                .iter()
                .filter_map(|(component, export)| {
                    export.then_some(*element.stress().index(component))
                })
                .map(|value| value.to_string())
                .chain(iter::once(element.is_broken().to_string()))
                .collect::<Vec<_>>()
        })
        .try_for_each(|record| writer.write_record(record))?;
    Ok(())
}
