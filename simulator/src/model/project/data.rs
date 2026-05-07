use crate::model::{boundary_conditions::BoundaryConditions, PolygonData};
use cpd::ExportData;
use mesh::Mesh;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WithShape {
    pub polygon_data: PolygonData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithBoundaryConditions {
    pub polygon_data: PolygonData,
    pub boundary_conditions: BoundaryConditions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithMesh {
    pub polygon_data: PolygonData,
    pub boundary_conditions: BoundaryConditions,
    pub mesh: Arc<Mesh>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithCpdExportData {
    pub polygon_data: PolygonData,
    pub boundary_conditions: BoundaryConditions,
    pub mesh: Arc<Mesh>,
    pub cpd_export_data: ExportData,
}

#[sealed::sealed]
pub trait State {}

macro_rules! impl_state {
    ( $($s:ty),* ) => {
        $(
            #[sealed::sealed]
            impl State for $s {}
        )*
    };
}

impl_state!(
    WithShape,
    WithBoundaryConditions,
    WithMesh,
    WithCpdExportData
);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Data<S: State> {
    state: S,
}

impl<S: State> Data<S> {
    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut S {
        &mut self.state
    }

    pub fn take_state(self) -> S {
        self.state
    }
}

impl Data<WithShape> {
    pub fn override_shape(self, polygon_data: PolygonData) -> Self {
        Self {
            state: WithShape { polygon_data },
        }
    }

    pub fn with_boundary_conditions(
        self,
        boundary_conditions: BoundaryConditions,
    ) -> Data<WithBoundaryConditions> {
        Data {
            state: WithBoundaryConditions {
                polygon_data: self.state.polygon_data,
                boundary_conditions,
            },
        }
    }
}

impl Data<WithBoundaryConditions> {
    pub fn without_boundary_conditions(self) -> (Data<WithShape>, BoundaryConditions) {
        (
            Data {
                state: WithShape {
                    polygon_data: self.state.polygon_data,
                },
            },
            self.state.boundary_conditions,
        )
    }

    pub fn with_mesh(self, mesh: Arc<Mesh>) -> Data<WithMesh> {
        Data {
            state: WithMesh {
                polygon_data: self.state.polygon_data,
                boundary_conditions: self.state.boundary_conditions,
                mesh,
            },
        }
    }
}

impl Data<WithMesh> {
    pub fn without_mesh(self) -> (Data<WithBoundaryConditions>, Arc<Mesh>) {
        (
            Data {
                state: WithBoundaryConditions {
                    polygon_data: self.state.polygon_data,
                    boundary_conditions: self.state.boundary_conditions,
                },
            },
            self.state.mesh,
        )
    }

    pub fn with_export_data(self, export_data: ExportData) -> Data<WithCpdExportData> {
        Data {
            state: WithCpdExportData {
                polygon_data: self.state.polygon_data,
                boundary_conditions: self.state.boundary_conditions,
                mesh: self.state.mesh,
                cpd_export_data: export_data,
            },
        }
    }
}

impl Data<WithCpdExportData> {
    pub fn without_export_data(self) -> (Data<WithMesh>, ExportData) {
        (
            Data {
                state: WithMesh {
                    polygon_data: self.state.polygon_data,
                    boundary_conditions: self.state.boundary_conditions,
                    mesh: self.state.mesh,
                },
            },
            self.state.cpd_export_data,
        )
    }
}
