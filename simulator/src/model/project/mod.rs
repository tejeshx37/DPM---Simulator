pub mod data;
mod manager;
mod workspace;

pub use manager::{ClosedHandle, Manager, OpenHandle, RecentHandle, UntitledHandle};
pub use workspace::Workspace;

use super::storage;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::{
    ffi::{OsStr, OsString},
    fs,
    hash::{Hash, Hasher},
    io::{self, ErrorKind},
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

lazy_static::lazy_static! {
    pub static ref PROJECT_FILE_EXT: &'static OsStr = OsStr::new("simproj");
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Closed;

#[derive(Debug, Serialize, Deserialize)]
pub struct Untitled<D>(D);

impl<D> Deref for Untitled<D> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<D> DerefMut for Untitled<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Open<D>(D);

impl<D> Deref for Open<D> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<D> DerefMut for Open<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[sealed::sealed]
pub trait State {}

#[sealed::sealed]
impl<D> State for Open<D> {}

#[sealed::sealed]
impl<D> State for Untitled<D> {}

#[sealed::sealed]
impl State for Closed {}

#[derive(Debug, Serialize, Deserialize, Getters)]
pub struct Project<S: State> {
    name: OsString,
    workspace_path: PathBuf,
    state: S,
}

impl<S: State> Hash for Project<S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.workspace_path.hash(state);
    }
}

impl<S: State> Project<S> {
    pub fn state_mut(&mut self) -> &mut S {
        &mut self.state
    }

    pub fn delete(self) -> Result<(), (Self, io::Error)> {
        let path = self.path();
        if path.is_file() {
            match fs::remove_file(path) {
                Ok(()) => Ok(()),
                Err(err) => Err((self, err)),
            }
        } else {
            Ok(())
        }
    }

    fn path(&self) -> PathBuf {
        Self::create_path(&self.workspace_path, &self.name)
    }

    fn create_path(workspace_path: &Path, name: &OsStr) -> PathBuf {
        workspace_path.join(name).with_extension(*PROJECT_FILE_EXT)
    }

    fn no_dir_exists(path: &Path) -> io::Result<()> {
        if path.is_dir() {
            Err(io::Error::new(
                ErrorKind::AlreadyExists,
                format!(
                    "A directory with the same path {} already exists",
                    path.display()
                ),
            ))
        } else {
            Ok(())
        }
    }

    fn no_dir_or_file_exists(path: &Path) -> io::Result<()> {
        Self::no_dir_exists(path)?;
        if path.is_file() {
            Err(io::Error::new(
                ErrorKind::AlreadyExists,
                format!("A file already exists at path {}", path.display()),
            ))
        } else {
            Ok(())
        }
    }
}

impl Project<Closed> {
    fn open<D>(self) -> Result<Project<Open<D>>, (Self, io::Error)>
    where
        D: for<'de> Deserialize<'de>,
    {
        match storage::read_data_in_path(self.path()) {
            Ok(state) => Ok(Project {
                name: self.name,
                workspace_path: self.workspace_path,
                state,
            }),
            Err(err) => Err((self, err)),
        }
    }
}

impl<D> Project<Open<D>> {
    pub fn save(&mut self) -> io::Result<()>
    where
        D: Serialize,
    {
        let path = self.path();
        Self::no_dir_exists(&path)?;
        storage::save_data_to_path(path, &self.state)
    }

    fn save_at_path(&mut self, path: PathBuf) -> io::Result<()>
    where
        D: Serialize,
    {
        self.name = name_from_project_path(&path)?;
        self.workspace_path = workspace_path_from_project_path(&path)?;
        let path = Self::create_path(&self.workspace_path, &self.name);
        Self::no_dir_or_file_exists(&path)
            .and_then(|()| storage::save_data_to_path(path, &self.state))
    }

    fn close(self) -> Project<Closed> {
        Project {
            name: self.name,
            workspace_path: self.workspace_path,
            state: Closed,
        }
    }

    fn closed(&self) -> Project<Closed> {
        Project {
            name: self.name.clone(),
            workspace_path: self.workspace_path.clone(),
            state: Closed,
        }
    }
}

impl<D> Project<Untitled<D>> {
    pub(super) fn set_path<P>(self, path: P) -> Result<Project<Open<D>>, (Self, io::Error)>
    where
        P: AsRef<Path>,
    {
        let name = match name_from_project_path(path.as_ref()) {
            Ok(name) => name,
            Err(err) => return Err((self, err)),
        };
        let workspace_path = match workspace_path_from_project_path(path.as_ref()) {
            Ok(workspace_path) => workspace_path,
            Err(err) => return Err((self, err)),
        };
        let path = Self::create_path(&workspace_path, &name);
        match Self::no_dir_or_file_exists(&path) {
            Ok(()) => Ok(Project {
                name,
                workspace_path,
                state: Open(self.state.0),
            }),
            Err(err) => Err((self, err)),
        }
    }
}

fn has_valid_extension(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == *PROJECT_FILE_EXT)
}

fn name_from_project_path(path: &Path) -> io::Result<OsString> {
    path.file_stem()
        .map(|stem| stem.to_os_string())
        .ok_or_else(|| {
            io::Error::new(
                ErrorKind::Other,
                format!("Unable to get file name from path {}", path.display()),
            )
        })
}

fn workspace_path_from_project_path(path: &Path) -> io::Result<PathBuf> {
    path.parent()
        .map(|path| path.to_path_buf())
        .ok_or(io::Error::new(
            ErrorKind::Other,
            "Cannot have root folder as workspace",
        ))
}
