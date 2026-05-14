use super::{
    project::data::{Data, WithBoundaryConditions, WithMesh},
    state_channel, PolygonData, RefreshToken,
};
use cgal::{PolygonSet, PolygonSetInput};
use mesh::{Callback, Mesh, SeedingConfig};
use std::{
    sync::{mpsc, Arc},
    thread::{self, JoinHandle},
};

#[derive(Debug, Default, Clone)]
pub enum State {
    #[default]
    Idle,
    GeneratingMesh(mesh::State),
    Mesh(Arc<Mesh>),
}

#[derive(Debug)]
pub struct MeshGenerator<T: RefreshToken> {
    project_data: Data<WithBoundaryConditions>,
    worker: Worker<T>,
    refresh_token_set: bool,
}

impl<T: RefreshToken> MeshGenerator<T> {
    pub fn new(
        project_data: Data<WithBoundaryConditions>,
        state_sender: state_channel::Sender<State>,
        error_sender: state_channel::Sender<String>,
    ) -> Self {
        Self {
            project_data,
            worker: Worker::new(state_sender, error_sender),
            refresh_token_set: false,
        }
    }

    pub fn new_with_mesh(
        project_data: Data<WithMesh>,
        state_sender: state_channel::Sender<State>,
        error_sender: state_channel::Sender<String>,
    ) -> Result<Self, String> {
        let (project_data, mesh) = project_data.without_mesh();
        if state_sender.send(State::Mesh(mesh)).is_err() {
            Err(String::from("State channel is already dropped"))
        } else {
            Ok(Self {
                project_data,
                worker: Worker::new(state_sender, error_sender),
                refresh_token_set: false,
            })
        }
    }

    pub fn set_refresh_token(&mut self, refresh_token: impl Into<T>) {
        if self.refresh_token_set {
            return;
        }
        self.worker
            .send(Command::SetRefreshToken(refresh_token.into()));
        self.refresh_token_set = true;
    }

    pub fn polygon_data(&self) -> &PolygonData {
        &self.project_data.state().polygon_data
    }

    pub fn project_data(&self) -> &Data<WithBoundaryConditions> {
        &self.project_data
    }

    pub fn project_data_with_bc(self) -> Data<WithBoundaryConditions> {
        self.project_data
    }

    pub fn generate(
        &mut self,
        num_points: u32,
        size_bound_override: Option<f64>,
        thickness: f64,
        seeding_config: Option<SeedingConfig>,
    ) -> Result<(), String> {
        if num_points == 0 {
            return Err(String::from("Number of points must be greater than 0"));
        }
        if num_points > 10000 {
            return Err(String::from("Number of points is too high (max 10,000)"));
        }
        self.worker.send(Command::Input(Input {
            polygon_set_inputs: self.polygon_data().inputs.clone(),
            num_points,
            size_bound_override,
            thickness,
            seeding_config,
        }));
        Ok(())
    }
}

#[derive(Debug)]
enum Command<T: RefreshToken> {
    SetRefreshToken(T),
    Input(Input),
}

#[derive(Debug)]
struct Input {
    polygon_set_inputs: Vec<PolygonSetInput>,
    num_points: u32,
    size_bound_override: Option<f64>,
    thickness: f64,
    seeding_config: Option<SeedingConfig>,
}

#[derive(Debug)]
struct Worker<T: RefreshToken> {
    command_sender: Option<mpsc::Sender<Command<T>>>,
    handle: Option<JoinHandle<()>>,
}

impl<T: RefreshToken> Worker<T> {
    fn new(
        state_sender: state_channel::Sender<State>,
        error_sender: state_channel::Sender<String>,
    ) -> Self {
        let (command_sender, command_receiver) = mpsc::channel();
        Self {
            command_sender: Some(command_sender),
            handle: {
                Some(thread::spawn(move || {
                    Self::command_queue(command_receiver, T::default(), state_sender, error_sender)
                }))
            },
        }
    }

    fn send(&mut self, command: Command<T>) {
        self.command_sender
            .as_ref()
            .expect("Sender is present")
            .send(command)
            .unwrap_or_else(|_| {
                let err = self
                    .handle
                    .take()
                    .expect("Handle should be present")
                    .join()
                    .expect_err("Sender should be dropped only if worker thread has panicked");
                std::panic::resume_unwind(err)
            })
    }

    fn command_queue(
        command_receiver: mpsc::Receiver<Command<T>>,
        mut refresh_token: T,
        state_sender: state_channel::Sender<State>,
        error_sender: state_channel::Sender<String>,
    ) {
        while let Ok(command) = command_receiver.recv() {
            let input = match command {
                Command::SetRefreshToken(token) => {
                    refresh_token = token;
                    continue;
                }
                Command::Input(input) => input,
            };
            macro_rules! send_state_discard_err {
                ( $state:expr ) => {
                    if state_sender.send($state).is_ok() {
                        refresh_token.refresh();
                    }
                };
            }
            macro_rules! send_state {
                ( $state:expr ) => {
                    if state_sender.send($state).is_err() {
                        break;
                    }
                    refresh_token.refresh();
                };
            }
            macro_rules! send_err {
                ( $err:expr ) => {{
                    if error_sender.send($err).is_err() {
                        break;
                    }
                    refresh_token.refresh();
                }};
            }
            let polyhedron_set = match cgal::PolyhedronSet::from_inputs(&input.polygon_set_inputs) {
                Ok(ps) => ps,
                Err(err) => {
                    send_err!(err);
                    continue;
                }
            };
            
            let result = if polyhedron_set.get_vertices().len() > 0 {
                Mesh::generate_from_polyhedron(
                    &polyhedron_set,
                    input.num_points,
                    input.size_bound_override,
                    input.seeding_config.clone(),
                    Callback::from(|state| send_state_discard_err!(State::GeneratingMesh(state))),
                )
            } else {
                let polygon_set = match PolygonSet::from_inputs(&input.polygon_set_inputs) {
                    Ok(ps) => ps,
                    Err(err) => {
                        send_err!(err);
                        continue;
                    }
                };
                let polygons = polygon_set.polygon_with_holes();
                if polygons.is_empty() {
                    send_err!(String::from("No polygons found in the current configuration. Please ensure you have drawn at least one shape."));
                    continue;
                }
                let primitive = if input.polygon_set_inputs.len() == 1 {
                    match &input.polygon_set_inputs[0] {
                        cgal::PolygonSetInput::Join(kind) => Some(kind),
                        _ => None,
                    }
                } else {
                    None
                };

                Mesh::generate(
                    &polygons[0],
                    input.num_points,
                    input.size_bound_override,
                    input.thickness,
                    primitive,
                    input.seeding_config,
                    Callback::from(|state| send_state_discard_err!(State::GeneratingMesh(state))),
                )
            };

            match result {
                Ok(mesh) => {
                    send_state!(State::Mesh(Arc::new(mesh)));
                }
                Err(err) => {
                    send_err!(err);
                }
            }
        }
    }
}

impl<T: RefreshToken> Drop for Worker<T> {
    fn drop(&mut self) {
        // Drop the sender first — this signals the background thread to
        // exit its blocking recv() loop.
        drop(self.command_sender.take());
        // Now join the thread so we wait until all CGAL work is done
        // before the main thread proceeds and potentially accesses CGAL
        // from a new thread. This prevents the null CORE_algebraic_number_traits crash.
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
