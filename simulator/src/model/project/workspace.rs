use super::storage::DATA_DIR_FOLDER;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Workspace {
    path: PathBuf,
    relative_path: PathBuf,
}

impl PartialEq for Workspace {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for Workspace {}

impl Workspace {
    pub fn new(home_dir: &Path, path: PathBuf) -> io::Result<Self> {
        puffin::profile_function!();
        if !path.is_absolute() {
            return Err(io::Error::new(
                ErrorKind::Other,
                format!("Path {} is not absolute", path.display()),
            ));
        }
        fs::create_dir_all(&path)?;
        let relative_path =
            pathdiff::diff_paths(&path, home_dir).expect("Relative path should be present");
        Ok(Self {
            path,
            relative_path,
        })
    }

    pub fn default(home_dir: &Path) -> io::Result<Self> {
        Self::new(home_dir, home_dir.join(DATA_DIR_FOLDER))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Display for Workspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.relative_path.display().fmt(f)
    }
}
