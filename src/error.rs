use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Unable to read file: {0:?}")]
    FileReadError(PathBuf),

    #[error("output_dir must be an existing directory: {0:?}")]
    NotADirectoryError(PathBuf),

    #[error("unknown data store error")]
    Unknown,
}
