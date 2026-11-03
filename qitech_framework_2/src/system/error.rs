use std::io;
use std::path::PathBuf;

use qitech_framework_core::system::SystemError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SystemLoadError {
    #[error("failed to read `{}`: {source}", path.display())]
    Read { path: PathBuf, source: io::Error },

    #[error("invalid system file:\n{0}")]
    Parse(#[from] SystemError),

    /// The file parsed, but does not match what was registered in Rust.
    #[error("invalid system:\n{}", .0.join("\n"))]
    Resolve(Vec<String>),
}
