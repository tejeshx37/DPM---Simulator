use std::{
    ops::Deref,
    sync::mpsc::{self, TryRecvError},
};

#[derive(Debug)]
pub struct Sender<T>(mpsc::SyncSender<T>);

impl<T> Deref for Sender<T> {
    type Target = mpsc::SyncSender<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct Receiver<T, D>
where
    T: Into<D>,
{
    pub data: D,
    receiver: mpsc::Receiver<T>,
}

pub type STReceiver<T> = Receiver<T, T>;

#[derive(Debug)]
pub struct DisconnectedError;

impl<T, D> Receiver<T, D>
where
    T: Into<D>,
{
    pub fn update(&mut self) -> Result<(), DisconnectedError> {
        match self.receiver.try_recv() {
            Ok(value) => {
                self.data = value.into();
                Ok(())
            }
            Err(err) => match err {
                TryRecvError::Empty => Ok(()),
                TryRecvError::Disconnected => Err(DisconnectedError),
            },
        }
    }

    pub fn update_and_get(&mut self) -> Result<&D, DisconnectedError> {
        self.update().map(|()| &self.data)
    }
}

fn mapped_channel<T, D>(data: D, bounds: usize) -> (Sender<T>, Receiver<T, D>)
where
    T: Into<D>,
{
    let (sender, receiver) = mpsc::sync_channel(bounds);
    (Sender(sender), Receiver { data, receiver })
}

pub fn with_default<T, D>(bounds: usize) -> (Sender<T>, Receiver<T, D>)
where
    D: Default,
    T: Into<D>,
{
    mapped_channel(D::default(), bounds)
}

pub fn same_type_with_default<T: Default>(bounds: usize) -> (Sender<T>, Receiver<T, T>) {
    with_default(bounds)
}
