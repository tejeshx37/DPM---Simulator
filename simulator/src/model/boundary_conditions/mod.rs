use super::{
    project::data::{Data, WithBoundaryConditions, WithShape},
    PolygonData,
};
use cgal::{BoundaryId, Coordinate, PolygonSetInput};
use cpd::boundary_condition::BoundaryCondition;
use fxhash::FxHashMap;
use serde::{Deserialize, Serialize};

pub type BoundaryConditions = FxHashMap<BoundaryId, BoundaryCondition>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Configurator {
    project_data: Data<WithShape>,
    boundary_conditions: BoundaryConditions,
}

impl From<Data<WithShape>> for Configurator {
    fn from(project_data: Data<WithShape>) -> Self {
        Self {
            boundary_conditions: Self::default_boundary_conditions(
                &project_data.state().polygon_data,
            ),
            project_data,
        }
    }
}

impl From<Data<WithBoundaryConditions>> for Configurator {
    fn from(project_data: Data<WithBoundaryConditions>) -> Self {
        let (project_data, boundary_conditions) = project_data.without_boundary_conditions();
        Self {
            boundary_conditions,
            project_data,
        }
    }
}

impl Configurator {
    fn default_boundary_conditions(polygon_data: &PolygonData) -> BoundaryConditions {
        puffin::profile_function!();
        polygon_data.polygon_set().polygon_with_holes()[0]
            .boundaries_iter()
            .map(|(id, _)| (id, BoundaryCondition::default()))
            .collect()
    }

    pub fn first_boundary_id(&self) -> BoundaryId {
        puffin::profile_function!();
        self.polygon_data().polygon_set().polygon_with_holes()[0]
            .boundaries_iter()
            .next()
            .expect("Non empty boundaries")
            .0
    }

    pub fn polygon_data(&self) -> &PolygonData {
        &self.project_data.state().polygon_data
    }

    pub fn project_data_with_shape(mut self) -> Data<WithShape> {
        puffin::profile_function!();
        let data = &mut self.project_data.state_mut().polygon_data;
        data.inputs.retain(|input| {
            !matches!(
                input,
                PolygonSetInput::Split {
                    boundary_id: _,
                    coordinate: _
                }
            )
        });
        self.project_data
    }

    pub fn project_data_with_bc(self) -> Data<WithBoundaryConditions> {
        self.project_data
            .with_boundary_conditions(self.boundary_conditions)
    }

    pub fn project_data_with_bc_cloned(&self) -> Data<WithBoundaryConditions> {
        puffin::profile_function!();
        self.project_data
            .clone()
            .with_boundary_conditions(self.boundary_conditions.clone())
    }

    pub fn get_condition(&self, id: &BoundaryId) -> Option<&BoundaryCondition> {
        self.boundary_conditions.get(id)
    }

    pub fn set_condition(&mut self, id: BoundaryId, condition: BoundaryCondition) {
        self.boundary_conditions
            .entry(id)
            .and_modify(|c| *c = condition);
    }

    pub fn split_curve(&mut self, boundary_id: BoundaryId, coordinate: Coordinate) {
        puffin::profile_function!();
        let input = PolygonSetInput::Split {
            boundary_id,
            coordinate,
        };
        self.project_data
            .state_mut()
            .polygon_data
            .inputs
            .push(input);
        self.boundary_conditions = Self::default_boundary_conditions(self.polygon_data());
    }
}
