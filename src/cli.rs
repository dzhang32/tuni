//! Module containing cli that parses and checks input arguments.

use crate::error::CliError;
use clap::{ArgAction, Parser};
use std::{fs, fs::File, path::PathBuf};

/// Parse and check input arguments.
#[derive(Parser)]
#[command(version, about = "tuni: Unify transcripts across different samples")]
pub struct Cli {
    /// A text file containing GTF/GFF paths.
    #[arg(short, long, value_name = "*.txt", required = true)]
    pub gtf_gff_path: PathBuf,

    /// Directory where outputted GTF/GFFs will be stored.
    #[arg(
        short,
        long,
        value_name = "/output/dir/",
        required = true,
        value_parser = Cli::parse_output_dir
    )]
    pub output_dir: PathBuf,

    /// Print log messages.
    #[arg(
        short,
        long,
        action = ArgAction::SetTrue,
    )]
    pub verbose: bool,
}

impl Cli {
    /// Parse file containing GTF/GFFs paths.
    ///
    /// Returns GTF/GFF paths on success, otherwise returns an error.
    ///
    /// # Errors
    ///
    /// Returns [`FileReadError`](CliError::FileReadError) if the file
    /// containing GTF/GFFs or any of the GTF/GFFs cannot be read.
    ///
    /// Returns [`FileEmptyError`](CliError::FileEmptyError) if the file
    /// containing GTF/GFFs is empty.
    ///
    /// Returns [`GtfGffParseError`](CliError::GtfGffParseError) if any of the GTF/GFFs
    /// do not exist or do not have the extension ".gtf"/".gff".
    pub fn parse_gtf_gff_paths(gtf_gff_path: PathBuf) -> Result<(String, Vec<PathBuf>), CliError> {
        let gtf_gff_paths = fs::read_to_string(&gtf_gff_path)
            .map_err(|_| CliError::FileReadError(gtf_gff_path.clone()))?
            .lines()
            .map(PathBuf::from)
            .collect::<Vec<PathBuf>>();

        if gtf_gff_paths.is_empty() {
            return Err(CliError::FileEmptyError(gtf_gff_path.clone()));
        }

        let gtf_gff_extension = gtf_gff_paths[0]
            .extension()
            .ok_or(CliError::GtfGffParseError(gtf_gff_paths[0].clone()))?;

        if gtf_gff_extension != "gtf" && gtf_gff_extension != "gff" {
            return Err(CliError::GtfGffParseError(gtf_gff_path.clone()));
        }

        for gtf_gff_path in &gtf_gff_paths {
            // Make sure all GTF/GFFs have the same extension.
            if !gtf_gff_path.is_file()
                || !gtf_gff_path
                    .extension()
                    .is_some_and(|x| x == gtf_gff_extension)
            {
                return Err(CliError::GtfGffParseError(gtf_gff_path.clone()));
            }
            // open() will return an error if the file is unreadable e.g. due to permissions.
            File::open(gtf_gff_path).map_err(|_| CliError::FileReadError(gtf_gff_path.clone()))?;
        }

        // gtf_gff_extension has been checked above to be be "gtf"/"gff".
        Ok((
            gtf_gff_extension.to_os_string().into_string().unwrap(),
            gtf_gff_paths,
        ))
    }

    /// Parse output directory.
    ///
    /// Returns output directory path on success, otherwise returns an error.
    ///
    /// # Errors
    ///
    /// Returns [`NotADirectoryError`](CliError::NotADirectoryError) if any of the GTF/GFFs
    /// do not exist or do not have the extension ".gtf"/".gff".
    fn parse_output_dir(s: &str) -> Result<PathBuf, CliError> {
        let output_dir = PathBuf::from(s);
        if !output_dir.is_dir() {
            return Err(CliError::NotADirectoryError(output_dir));
        };
        Ok(output_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test that cli will error if GTF/GFF paths
    // 1. don't exist, 2. are not readable or lack the ".gtf"/".gff" extension.
    #[test]
    fn test_parse_gtf_paths() {
        let result = Cli::parse_gtf_gff_paths(PathBuf::from("does_not_exist.txt"));
        assert!(result.is_err_and(|e| e.to_string().contains("Unable to read file")));

        let result = Cli::parse_gtf_gff_paths(PathBuf::from("tests/data/unit/gtf_paths_empty.txt"));
        assert!(result.is_err_and(|e| e.to_string().contains("is empty")));

        let result =
            Cli::parse_gtf_gff_paths(PathBuf::from("tests/data/unit/gtf_paths_missing_gtf.txt"));
        assert!(result.is_err_and(|e| e
            .to_string()
            .contains("GTF/GFFs must be readable and all have the same extension")));

        let result =
            Cli::parse_gtf_gff_paths(PathBuf::from("tests/data/unit/gtf_paths_includes_gff.txt"));
        assert!(result.is_err_and(|e| e.to_string().contains("all have the same extension")));

        let result = Cli::parse_gtf_gff_paths(PathBuf::from("tests/data/unit/gtf_paths.txt"));
        assert!(result.is_ok());
    }

    /// Test that cli will error if output_dir is not an existing directory.
    #[test]
    fn test_parse_output_dir() {
        let result = Cli::parse_output_dir("/does/not/exist/");
        assert!(result.is_err_and(|e| e
            .to_string()
            .contains("output_dir must be an existing directory")));

        // Not a directory.
        let result = Cli::parse_output_dir("tests/data/unit/gtf_paths_missing_gtf.txt");
        assert!(result.is_err_and(|e| e
            .to_string()
            .contains("output_dir must be an existing directory")));

        let result = Cli::parse_output_dir("tests/data/unit/");
        assert!(result.is_ok());
    }
}
