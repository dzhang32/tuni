//! Module containing cli that parses and checks input arguments. 

use clap::{ArgAction, Parser};
use std::{fs, fs::File, path::PathBuf};
use crate::error::CliError;

/// Parse and check input arguments.
#[derive(Parser)]
#[command(
    version, 
    about = "tuni: Unify transcripts outputted by transcript assembly tools"
)]
pub struct Cli {
    /// A text file containing GTF paths.
    #[arg(
        short, 
        long, 
        value_name = "*.txt", 
        required = true,
    )]
    pub gtf_paths: PathBuf,

    /// Directory where outputted GTFs with unified transcripts will be stored.
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
    /// Parse file containing GTFs paths.
    /// 
    /// Returns GTF paths on success, otherwise returns an error.
    /// 
    /// # Errors 
    /// 
    /// Returns [`FileReadError`](CliError::FileReadError) if the file 
    /// containing GTFs or any of the GTFs cannot be read.
    /// 
    /// Returns [`GtfParseError`](CliError::GtfParseError) if any of the GTFs 
    /// do not exist or do not have the extension "gtf".
    pub fn parse_gtf_paths(gtf_paths: PathBuf) -> Result<Vec<PathBuf>, CliError> {
        let gtf_paths = fs::read_to_string(&gtf_paths)
            .map_err(|_| CliError::FileReadError(gtf_paths))?
            .lines()
            .map(PathBuf::from)
            .collect::<Vec<PathBuf>>();

        for gtf_path in &gtf_paths {
            if !gtf_path.is_file() || !gtf_path.extension().is_some_and(|x| x == "gtf") {
                return Err(CliError::GtfParseError(gtf_path.clone()))
            }
            // open() will return an error if the file is unreadable e.g. due to permissions.
            File::open(gtf_path).map_err(|_| CliError::FileReadError(gtf_path.clone()))?;
        }

        Ok(gtf_paths)
    }

    /// Parse output directory.
    /// 
    /// Returns output directory path on success, otherwise returns an error.
    /// 
    /// # Errors
    /// 
    /// Returns [`NotADirectoryError`](CliError::NotADirectoryError) if any of the GTFs 
    /// do not exist or do not have the extension "gtf".
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

    // Test that cli will error if GTF paths
    // 1. don't exist, 2. are not readable or lack the ".gtf" extension.
    #[test]
    fn test_parse_gtf_paths() {
        let result = Cli::parse_gtf_paths(PathBuf::from("does_not_exist.txt"));
        assert!(result.is_err_and(|e| e.to_string().contains("Unable to read file")));

        let result =
            Cli::parse_gtf_paths(PathBuf::from("tests/data/unit/gtf_paths_missing_gtf.txt"));
        assert!(result.is_err_and(|e| e.to_string().contains("GTFs must be a file with the '.gtf' extension")));

        let result = Cli::parse_gtf_paths(PathBuf::from("tests/data/unit/gtf_paths.txt"));
        assert!(result.is_ok());
    }

    /// Test that cli will error if output_dir is not an existing directory.
    #[test]
    fn test_parse_output_dir() {
        let result = Cli::parse_output_dir("/does/not/exist/");
        assert!(result.is_err_and(|e| e.to_string().contains("output_dir must be an existing directory")));

        // Not a directory.
        let result = Cli::parse_output_dir("tests/data/unit/gtf_paths_missing_gtf.txt");
        assert!(result.is_err_and(|e| e.to_string().contains("output_dir must be an existing directory")));

        let result = Cli::parse_output_dir("tests/data/unit/");
        assert!(result.is_ok());
    }
}