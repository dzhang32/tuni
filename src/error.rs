use std::path::PathBuf;
use thiserror::Error;

#[allow(clippy::enum_variant_names)]
#[derive(Error, Debug)]
pub enum CliError {
    #[error("FileReadError: Unable to read file {0:?}")]
    FileReadError(PathBuf),

    #[error("GtfParseError: GTFs must be a file with the '.gtf' extension, found {0:?}")]
    GtfParseError(PathBuf),

    #[error("NotADirectoryError: output_dir must be an existing directory {0:?}")]
    NotADirectoryError(PathBuf),
}

#[allow(clippy::enum_variant_names)]
#[derive(Error, Debug)]
pub enum GtfError {
    #[error("MissingTranscriptIdError: No transcript_id found in line {0:?}")]
    MissingTranscriptIdError(String),

    #[error("UnknownFeatureError: Feature must be 'exon' or 'CDS', found {0:?}.")]
    UnknownFeatureError(String),

    #[error("LineReadError: Unable to read line in {0:?}")]
    LineReadError(PathBuf),

    #[error("FileCreateError: Unable to create output file {0:?}")]
    FileCreateError(PathBuf),
}
