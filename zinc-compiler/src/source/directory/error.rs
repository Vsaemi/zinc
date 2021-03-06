//!
//! The source code directory error.
//!

use std::fmt;
use std::io;

///
/// The source code directory error.
///
#[derive(Debug)]
pub enum Error {
    /// The directory opening error.
    Reading(io::Error),
    /// The directory name getting error.
    StemNotFound,
    /// The directory entry getting error.
    DirectoryEntry(io::Error),
    /// The module entry is in the root directory. Only the application entry allowed there.
    ModuleEntryInRoot,
    /// The application entry file is deeper than the root directory.
    ApplicationEntryBeyondRoot,
    /// The module entry not found.
    ModuleEntryNotFound,
    /// The application entry not found. Only for the root directory.
    ApplicationEntryNotFound,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reading(inner) => write!(f, "reading: `{}`", inner),
            Self::StemNotFound => write!(f, "directory name not found"),
            Self::DirectoryEntry(inner) => write!(f, "directory entry: `{}`", inner),
            Self::ModuleEntryInRoot => write!(
                f,
                "the module entry file `{}.{}` cannot be the application entry",
                zinc_const::file_name::MODULE_ENTRY,
                zinc_const::extension::SOURCE,
            ),
            Self::ApplicationEntryBeyondRoot => write!(
                f,
                "the application entry file `{}.{}` is beyond the source code root",
                zinc_const::file_name::APPLICATION_ENTRY,
                zinc_const::extension::SOURCE,
            ),
            Self::ApplicationEntryNotFound => write!(
                f,
                "the application entry file `{}.{}` is missing",
                zinc_const::file_name::APPLICATION_ENTRY,
                zinc_const::extension::SOURCE,
            ),
            Self::ModuleEntryNotFound => write!(
                f,
                "the module entry file `{}.{}` is missing",
                zinc_const::file_name::MODULE_ENTRY,
                zinc_const::extension::SOURCE,
            ),
        }
    }
}
