use crate::model::{
    project::data::{Data, WithShape},
    state_channel, PolygonData, RefreshToken,
};
use cgal::{PolygonSet, PolygonSetInput};
use std::{
    sync::mpsc,
    thread::{self, JoinHandle},
};
use uuid::Uuid;

#[derive(Debug, Default, Clone)]
pub struct Snapshot {
    id: Uuid,
    can_undo: bool,
    can_redo: bool,
    num_polygons: usize,
    pub inputs: Vec<PolygonSetInput>,
}

#[derive(Debug)]
pub enum State {
    Processing,
    Generated(Snapshot),
}

impl Default for State {
    fn default() -> Self {
        State::Generated(Snapshot::default())
    }
}

#[derive(Debug)]
pub struct Configurator<T: RefreshToken> {
    worker: Worker<T>,
    refresh_token_set: bool,
}

impl<T: RefreshToken> Configurator<T> {
    pub fn new(
        project_data: Data<WithShape>,
        state_sender: state_channel::Sender<State>,
        error_sender: state_channel::Sender<String>,
    ) -> Result<Self, String> {
        Worker::new(
            project_data.take_state().polygon_data,
            state_sender,
            error_sender,
        )
        .map(|worker| Self {
            worker,
            refresh_token_set: false,
        })
    }

    pub fn set_refresh_token(&mut self, refresh_token: impl Into<T>) {
        if self.refresh_token_set {
            return;
        }
        self.worker
            .send(Command::SetRefreshToken(refresh_token.into()));
        self.refresh_token_set = true;
    }

    pub fn join_or_diff(&mut self, input: PolygonSetInput) {
        self.worker.send(Command::Process(input))
    }

    pub fn undo(&mut self) {
        self.worker.send(Command::Undo)
    }

    pub fn redo(&mut self) {
        self.worker.send(Command::Redo)
    }

    pub fn reset(&mut self) {
        self.worker.send(Command::Reset)
    }
}

#[derive(Debug)]
enum Command<T: RefreshToken> {
    SetRefreshToken(T),
    Process(PolygonSetInput),
    Undo,
    Redo,
    Reset,
}

#[derive(Debug, Default)]
struct Stacks {
    forward: Vec<PolygonSetInput>,
    backward: Vec<PolygonSetInput>,
}

impl From<&Stacks> for Snapshot {
    fn from(stacks: &Stacks) -> Self {
        let num_polygons = PolygonSet::from_inputs(&stacks.forward)
            .map(|set| set.polygon_with_holes().len())
            .unwrap_or_default();
        Self {
            id: Uuid::new_v4(),
            can_undo: !stacks.forward.is_empty(),
            can_redo: !stacks.backward.is_empty(),
            num_polygons,
            inputs: stacks.forward.clone(),
        }
    }
}

impl Snapshot {
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn can_redo(&self) -> bool {
        self.can_redo
    }

    pub fn can_undo(&self) -> bool {
        self.can_undo
    }

    pub fn num_polygons(&self) -> usize {
        self.num_polygons
    }

    pub fn polygon_set(&self) -> PolygonSet {
        PolygonSet::from_inputs(&self.inputs).unwrap_or_else(|err| {
            log::error!("Invalid snapshot polygon inputs: {err}");
            PolygonSet::default()
        })
    }
}

impl From<Snapshot> for Data<WithShape> {
    fn from(value: Snapshot) -> Self {
        Self::default().override_shape(PolygonData::new(value.inputs))
    }
}

#[derive(Debug)]
struct Worker<T: RefreshToken> {
    command_sender: mpsc::Sender<Command<T>>,
    handle: Option<JoinHandle<()>>,
}

impl<T: RefreshToken> Worker<T> {
    fn new(
        polygon_data: PolygonData,
        state_sender: state_channel::Sender<State>,
        error_sender: state_channel::Sender<String>,
    ) -> Result<Self, String> {
        let (command_sender, command_receiver) = mpsc::channel();
        let stacks = Stacks {
            forward: polygon_data.inputs,
            backward: vec![],
        };
        let snapshot = Snapshot::from(&stacks);
        if state_sender.send(State::Generated(snapshot)).is_err() {
            return Err(String::from("State channel is already dropped"));
        }
        Ok(Worker {
            command_sender,
            handle: {
                Some(thread::spawn(move || {
                    Self::run(
                        stacks,
                        T::default(),
                        state_sender,
                        error_sender,
                        command_receiver,
                    )
                }))
            },
        })
    }

    fn run(
        mut stacks: Stacks,
        mut refresh_token: T,
        state_sender: state_channel::Sender<State>,
        error_sender: state_channel::Sender<String>,
        command_receiver: mpsc::Receiver<Command<T>>,
    ) {
        macro_rules! send_state {
            ( $state:expr ) => {{
                if state_sender.send($state).is_err() {
                    break;
                }
                refresh_token.refresh();
            }};
        }
        macro_rules! send_err {
            ( $err:expr ) => {{
                if error_sender.send($err).is_err() {
                    break;
                }
                refresh_token.refresh();
            }};
        }
        while let Ok(command) = command_receiver.recv() {
            match command {
                Command::SetRefreshToken(token) => {
                    refresh_token = token;
                }
                Command::Process(input) => {
                    send_state!(State::Processing);
                    let polygon_set = PolygonSet::from_inputs(&stacks.forward);
                    let mut polygon_set = match polygon_set {
                        Ok(set) => set,
                        Err(err) => {
                            send_err!(err);
                            continue;
                        }
                    };
                    match polygon_set.process_input(&input) {
                        Ok(()) => {
                            stacks.forward.push(input);
                            stacks.backward.clear();
                        }
                        Err(err) => send_err!(err),
                    }
                    send_state!(State::Generated(Snapshot::from(&stacks)));
                }
                Command::Undo => {
                    let Some(input) = stacks.forward.pop() else {
                        continue;
                    };
                    stacks.backward.push(input);
                    send_state!(State::Generated(Snapshot::from(&stacks)));
                }
                Command::Redo => {
                    let Some(input) = stacks.backward.pop() else {
                        continue;
                    };
                    send_state!(State::Processing);
                    let polygon_set = PolygonSet::from_inputs(&stacks.forward);
                    let mut polygon_set = match polygon_set {
                        Ok(set) => set,
                        Err(err) => {
                            send_err!(err);
                            continue;
                        }
                    };
                    match polygon_set.process_input(&input) {
                        Ok(()) => stacks.forward.push(input),
                        Err(err) => {
                            stacks.backward.push(input);
                            send_err!(err);
                        }
                    }
                    send_state!(State::Generated(Snapshot::from(&stacks)));
                }
                Command::Reset => {
                    stacks.forward.clear();
                    stacks.backward.clear();
                    send_state!(State::default());
                }
            };
        }
    }

    fn send(&mut self, command: Command<T>) {
        self.command_sender
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
}
