//!
//! The project `build` directory.
//!

pub mod bytecode;

use std::fs;
use std::path::PathBuf;

use crate::error::directory::Error as DirectoryError;

///
/// The project `build` directory.
///
pub struct Directory {}

impl Directory {
    ///
    /// If the `path` does not end with the directory subpath, appends the subpath to the `path`.
    ///
    pub fn path(path: &PathBuf) -> PathBuf {
        let mut path = path.to_owned();
        if path.is_dir() && !path.ends_with(zinc_const::directory::BUILD) {
            path.push(PathBuf::from(zinc_const::directory::BUILD));
        }
        path
    }

    ///
    /// Creates a directory with all its parent directories if it does not exist.
    ///
    pub fn create(path: &PathBuf) -> Result<(), DirectoryError> {
        fs::create_dir_all(&Self::path(path)).map_err(DirectoryError::Creating)
    }

    ///
    /// Removes the directory with all its child directories.
    ///
    pub fn remove(path: &PathBuf) -> Result<(), DirectoryError> {
        let mut path = path.to_owned();
        if path.is_dir() && !path.ends_with(zinc_const::directory::BUILD) {
            path.push(PathBuf::from(zinc_const::directory::BUILD));
        }

        if path.exists() {
            fs::remove_dir_all(&path).map_err(DirectoryError::Removing)?;
        }

        Ok(())
    }
}
