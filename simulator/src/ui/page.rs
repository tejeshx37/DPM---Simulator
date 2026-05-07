use super::{boundary_conditions, drawing, meshing, simulation};
use egui::Ui;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum Page {
    #[default]
    Invalid,
    Drawing(drawing::Page),
    BoundaryConditions(boundary_conditions::Page),
    Meshing(meshing::Page),
    Simulation(simulation::Page),
}

impl Page {
    pub fn drawing() -> Self {
        Self::Drawing(drawing::Page::default())
    }

    pub fn add_menu_items(self, ui: &mut Ui) -> Self {
        match self {
            Self::Invalid => unreachable!(),
            Self::Drawing(mut page) => {
                page.add_menu_items(ui);
                Self::Drawing(page)
            }
            Self::BoundaryConditions(page) => {
                use boundary_conditions::MenuResponse;
                match page.add_menu_items(ui) {
                    MenuResponse::Noop(page) => Self::BoundaryConditions(page),
                    MenuResponse::EditShape(project_data) => {
                        Self::Drawing(drawing::Page::from(project_data))
                    }
                }
            }
            Self::Meshing(page) => {
                use meshing::MenuResponse;
                match page.add_menu_items(ui) {
                    MenuResponse::Noop(page) => Self::Meshing(page),
                    MenuResponse::EditBoundaryConditions(project_data) => {
                        Self::BoundaryConditions(boundary_conditions::Page::from(project_data))
                    }
                    MenuResponse::EditShape(project_data) => {
                        Self::Drawing(drawing::Page::from(project_data))
                    }
                }
            }
            Self::Simulation(page) => {
                use simulation::MenuResponse;
                match page.add_menu_items(ui) {
                    MenuResponse::Noop(page) => Self::Simulation(page),
                    MenuResponse::EditMesh(project_data) => {
                        Self::Meshing(meshing::Page::from(project_data))
                    }
                    MenuResponse::EditBoundaryConditions(project_data) => {
                        Self::BoundaryConditions(boundary_conditions::Page::from(project_data))
                    }
                    MenuResponse::EditShape(project_data) => {
                        Self::Drawing(drawing::Page::from(project_data))
                    }
                }
            }
        }
    }

    pub fn add_contents(self, ui: &mut Ui, gpu_pipeline: Option<std::sync::Arc<cpd_wgpu::ComputePipeline>>) -> Self {
        match self {
            Self::Invalid => unreachable!(),
            Self::Drawing(page) => {
                use drawing::Response;
                match page.add_contents(ui) {
                    Response::Noop(page) => Self::Drawing(page),
                    Response::SetBoundaryConditions(data) => {
                        Self::BoundaryConditions(boundary_conditions::Page::from(data))
                    }
                }
            }
            Self::BoundaryConditions(page) => {
                use boundary_conditions::Response;
                match page.add_contents(ui) {
                    Response::Noop(page) => Self::BoundaryConditions(page),
                    Response::GenerateMesh(project_data) => {
                        Self::Meshing(meshing::Page::from(project_data))
                    }
                }
            }
            Self::Meshing(page) => {
                use meshing::Response;
                match page.add_contents(ui) {
                    Response::Noop(page) => Self::Meshing(page),
                    Response::RunSimulation(project_data) => {
                        Self::Simulation(simulation::Page::new(project_data, gpu_pipeline))
                    }
                }
            }
            Self::Simulation(page) => {
                use simulation::Response;
                match page.add_contents(ui) {
                    Response::Noop(page) => Self::Simulation(page),
                }
            }
        }
    }
}
