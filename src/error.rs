//! Custom error types returned by tuni.

use std::path::PathBuf;
use thiserror::Error;

/// Errors resulting from cli parsing.
#[allow(clippy::enum_variant_names)]
#[derive(Error, Debug)]
pub enum CliError {
    /// The file provided is unreadable.
    #[error("FileReadError: Unable to read file {0:?}")]
    FileReadError(PathBuf),

    /// The file containing GTF/GFF paths is empty.
    #[error("FileEmptyError: Provided file {0:?} is empty")]
    FileEmptyError(PathBuf),

    /// The GTF/GFFs include a file which is
    /// 1. not readable, 2. does not have a ".gtf"/".gff" extension or
    /// 3. has an extension distinct from the remaining GTF/GFFs.
    #[error(
        "GtfGffParseError: GTF/GFFs must be readable and all have the same extension ('.gtf' or '.gff'), found {0:?}"
    )]
    GtfGffParseError(PathBuf),

    /// The path does not point to a directory (e.g. it is a file).
    #[error("NotADirectoryError: output_dir must be an existing directory {0:?}")]
    NotADirectoryError(PathBuf),
}

/// Errors resulting from processing GTF/GFF lines.
#[allow(clippy::enum_variant_names)]
#[derive(Error, Debug)]
pub enum GtfGffError {
    /// The ("exon" or "CDS") record does not contain the "transcript_id" attribute.
    #[error("MissingTranscriptIdError: No transcript_id found in line {0:?}")]
    MissingTranscriptIdError(String),

    /// `tuni` should filter for only "exon"/"CDS" records. Therefore, if this
    /// error appears, it likely points to a tuni bug in filtering.
    #[error("UnknownFeatureError: Feature must be 'exon' or 'CDS', found {0:?}.")]
    UnknownFeatureError(String),

    /// `tuni` checks files have the "gtf"/"gff" extension at the cli parsing
    /// stage. Therefore, if this error appears, it likely points to a tuni bug
    /// in cli parsing.
    #[error("UnknownFeatureError: Feature must be 'exon' or 'CDS', found {0:?}.")]
    UnknownExtensionError(String),

    /// The line from the GTF/GFF could not be read.
    #[error("LineReadError: Unable to read line in {0:?}")]
    LineReadError(PathBuf),

    /// The file could not be created.
    #[error("FileCreateError: Unable to create output file {0:?}")]
    FileCreateError(PathBuf),

    /// Could not write to the file.
    #[error("FileWriteError: Unable to write line to {0:?}")]
    FileWriteError(PathBuf),
}
