use crate::{
    boundary_condition::{BoundaryCondition, Displacement},
    time_series_value::TimeStampedValue,
    TimeSeriesValue,
};
use cgal::triangulation::{Data as TriangulationData, Vertex};
use fxhash::FxHashMap;
use nalgebra::Vector3;
use rand::prelude::*;
use rand_distr::UnitBall;
use rayon::prelude::*;
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum Node {
    Interior(NodeData),
    OnBoundary(NodeData, BoundaryCondition),
}

impl Deref for Node {
    type Target = NodeData;

    fn deref(&self) -> &Self::Target {
        match self {
            Node::Interior(data) | Node::OnBoundary(data, _) => data,
        }
    }
}

impl DerefMut for Node {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Node::Interior(data) | Node::OnBoundary(data, _) => data,
        }
    }
}

impl Node {
    pub fn position_time_series(&self) -> &TimeSeriesValue<Vector3<f32>> {
        &self.position_time_series
    }

    pub fn force(&self) -> &Vector3<f32> {
        &self.force
    }

    pub fn velocity(&self) -> &Vector3<f32> {
        &self.velocity
    }

    pub(crate) fn reset(&mut self) {
        self.deref_mut().reset();
    }

    pub(crate) fn apply_force_and_bc(
        &mut self,
        force: Vector3<f32>,
        iterations: u128,
        damping_constant: f32,
        time_delta: f32,
    ) {
        // Numerical Safeguard: Sanitize force to prevent NaN propagation
        let force = if force.iter().all(|f| f.is_finite()) {
            force
        } else {
            Vector3::zeros()
        };
        self.force = force;

        macro_rules! velocity_delta {
            ($mass:expr, $force:expr, $velocity:expr) => {{
                // Numerical Safeguard: Ensure mass is sufficient for stability
                let mass = $mass.max(1e-6); 
                // Rayleigh mass-proportional damping: F_damp = alpha * mass * v
                let f_damp = $velocity * (damping_constant * mass);
                (($force - f_damp) * time_delta) / mass
            }};
        }
        let time = iterations as f32 * time_delta;
        macro_rules! update_pos_and_velocity {
            ($node:expr) => {{
                let v_delta = velocity_delta!($node.mass(), $node.force, $node.velocity);
                $node.velocity += v_delta;
                
                let position: Vector3<f32> = $node.position() + $node.velocity * time_delta;
                $node.position_time_series.set_or_push(time, position);
            }};
        }
        match self {
            Node::OnBoundary(node, BoundaryCondition::Displacement(displacement)) => {
                let mut position: Vector3<f32> = *node.position();
                macro_rules! update_pos_and_velocity_comp {
                    ( $comp:ident ) => {{
                        let v_delta = velocity_delta!(node.mass(), node.force.$comp, node.velocity.$comp);
                        node.velocity.$comp += v_delta;
                        position.$comp += node.velocity.$comp * time_delta;
                    }};
                }
                match &displacement {
                    Displacement::X(f) => {
                        if let Some(x) = f.of(time) {
                            position.x = node.initial_position.x + x;
                        }
                        node.velocity.y += velocity_delta!(node.mass(), node.force.y, node.velocity.y);
                        node.velocity.z += velocity_delta!(node.mass(), node.force.z, node.velocity.z);
                        position.y += node.velocity.y * time_delta;
                        position.z += node.velocity.z * time_delta;
                    }
                    Displacement::Y(f) => {
                        if let Some(y) = f.of(time) {
                            position.y = node.initial_position.y + y;
                        }
                        update_pos_and_velocity_comp!(x);
                        update_pos_and_velocity_comp!(z);
                    }
                    Displacement::Z(f) => {
                        if let Some(z) = f.of(time) {
                            position.z = node.initial_position.z + z;
                        }
                        update_pos_and_velocity_comp!(x);
                        update_pos_and_velocity_comp!(y);
                    }
                    Displacement::XY(vf) => {
                        if let Some(x) = vf.x.of(time) {
                            position.x = node.initial_position.x + x;
                        }
                        if let Some(y) = vf.y.of(time) {
                            position.y = node.initial_position.y + y;
                        }
                        update_pos_and_velocity_comp!(z);
                    }
                    Displacement::XZ(vf) => {
                        if let Some(x) = vf.x.of(time) {
                            position.x = node.initial_position.x + x;
                        }
                        if let Some(z) = vf.y.of(time) {
                            position.z = node.initial_position.z + z;
                        }
                        update_pos_and_velocity_comp!(y);
                    }
                    Displacement::YZ(vf) => {
                        if let Some(y) = vf.x.of(time) {
                            position.y = node.initial_position.y + y;
                        }
                        if let Some(z) = vf.y.of(time) {
                            position.z = node.initial_position.z + z;
                        }
                        update_pos_and_velocity_comp!(x);
                    }
                    Displacement::XYZ(vf) => {
                        if let Some(x) = vf.x.of(time) {
                            position.x = node.initial_position.x + x;
                        }
                        if let Some(y) = vf.y.of(time) {
                            position.y = node.initial_position.y + y;
                        }
                        if let Some(z) = vf.z.of(time) {
                            position.z = node.initial_position.z + z;
                        }
                    }
                }
                node.position_time_series.set_or_push(time, position);
            }
            Node::OnBoundary(node, BoundaryCondition::Force(external_force)) => {
                node.force.zip_apply(external_force, |fv, f| {
                    if let Some(v) = f.of(time) {
                        *fv += v;
                    }
                });
                update_pos_and_velocity!(self);
            }
            Node::OnBoundary(_, BoundaryCondition::Free) | Node::Interior(_) => {
                update_pos_and_velocity!(self);
            }
        };
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct NodeData {
    pub(crate) position_time_series: TimeSeriesValue<Vector3<f32>>,
    pub(crate) force: Vector3<f32>,
    pub(crate) velocity: Vector3<f32>,
    initial_velocity: Vector3<f32>,
    initial_position: Vector3<f32>,
    mass: f32,
}

impl NodeData {
    fn new(
        position: Vector3<f32>,
        mass: f32,
        initial_velocity: Vector3<f32>,
    ) -> Self {
        Self {
            position_time_series: TimeSeriesValue::Single(position),
            force: Vector3::zeros(),
            velocity: initial_velocity,
            initial_velocity,
            initial_position: position,
            mass,
        }
    }

    pub fn position(&self) -> &Vector3<f32> {
        self.position_time_series.latest()
    }

    pub fn initial_position(&self) -> &Vector3<f32> {
        &self.initial_position
    }

    pub fn mass(&self) -> f32 {
        self.mass
    }

    pub(crate) fn scale_mass(&mut self, scale: f32) {
        self.mass *= scale;
    }

    pub(crate) fn reset(&mut self) {
        self.force.x = 0.0;
        self.force.y = 0.0;
        self.force.z = 0.0;
        self.velocity = self.initial_velocity;
        match &mut self.position_time_series {
            TimeSeriesValue::Single(v) => *v = self.initial_position,
            TimeSeriesValue::Series(series) => {
                series.clear();
                series.push(TimeStampedValue {
                    time_stamp: 0.0,
                    value: self.initial_position,
                });
            }
        }
    }
}

fn cell_volume(index: usize, triangulation_data: &TriangulationData) -> f32 {
    let indices = triangulation_data.faces()[index].0;
    let vertices: &[Vertex] = triangulation_data.vertices();
    let point = |index: usize| vertices[indices[index]].point();
    let pq: Vector3<f32> = point(1) - point(0);
    let pr: Vector3<f32> = point(2) - point(0);
    let ps: Vector3<f32> = point(3) - point(0);
    (pq.dot(&pr.cross(&ps))).abs() / 6.0
}

fn incident_elements_volume(vertex: &Vertex, triangulation_data: &TriangulationData) -> f32 {
    vertex
        .incident_faces()
        .iter()
        .copied()
        .map(|index| cell_volume(index, triangulation_data))
        .sum()
}

fn random_velocity() -> Vector3<f32> {
    let mut rng = rand::thread_rng();
    let v: [f32; 3] = UnitBall.sample(&mut rng);
    Vector3::from(v) * 1e-4
}

pub fn nodes(
    triangulation_data: &TriangulationData,
    boundary_conditions: &FxHashMap<usize, BoundaryCondition>,
    density: f32,
) -> Box<[Node]> {
    triangulation_data
        .vertices()
        .par_iter()
        .enumerate()
        .map(|(i, vertex)| {
            let node_data = NodeData::new(
                *vertex.point(),
                density * incident_elements_volume(vertex, triangulation_data) / 4.0,
                random_velocity(),
            );
            match boundary_conditions.get(&i).cloned() {
                Some(bc) => Node::OnBoundary(node_data, bc),
                None => Node::Interior(node_data),
            }
        })
        .collect()
}
