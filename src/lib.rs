// TODO: should I switch these to use?
mod gtf;
mod unify;

use clap::Parser;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// A text file containing GTF paths.
    #[arg(
        short, 
        long, 
        value_name = "*.txt", 
        required = true, 
        value_parser = Cli::parse_gtf_paths
    )]
    gtf_paths: Vec<PathBuf>,

    /// Directory to store the outputted GTFs with unified transcripts.
    #[arg(
        short, 
        long, 
        value_name = "/output/dir/", 
        required = true, 
        value_parser = Cli::parse_output_path
    )]
    output_path: PathBuf,
}

impl Cli {
    /// Parse file containing GTFs, checking that GTFs exist and are readable.
    fn parse_gtf_paths(s: &str) -> Result<Vec<PathBuf>, String> {
        let gtf_paths_file = PathBuf::from(s);
        let gtf_paths = match fs::read_to_string(gtf_paths_file) {
            Ok(file) => file.lines().map(PathBuf::from).collect::<Vec<PathBuf>>(),
            Err(e) => return Err(format!("{s}: {e}")),
        };

        // Make sure all GTFs exist and are readable.
        for gtf_path in &gtf_paths {
            match File::open(gtf_path) {
                Ok(_) => {}
                Err(e) => return Err(format!("{}: {e}", gtf_path.display())),
            };
        }

        Ok(gtf_paths)
    }

    /// Parse output path, checking that it points to an existing directory.
    fn parse_output_path(s: &str) -> Result<PathBuf, String> {
        let output_path = PathBuf::from(s);
        if !output_path.is_dir() {
            return Err(format!(
                "output_path must point to an existing directory: {}",
                output_path.display()
            ));
        };
        Ok(output_path)
    }
}

pub fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that cli will error if file containing GTF paths and GTFs 1. don't
    /// exist or 2. are not readable.
    #[test]
    fn test_parse_gtf_paths() {
        let result = Cli::parse_gtf_paths("does_not_exist.txt");
        assert!(result.is_err(), "Expected an error, found: {:?}", result);

        let result = Cli::parse_gtf_paths("tests/data/gtf_paths_missing_gtf.txt");
        assert!(result.is_err(), "Expected an error, found: {:?}", result);

        let result = Cli::parse_gtf_paths("tests/data/gtf_paths.txt");
        assert!(result.is_ok(), "{:?}", result);
    }

    /// Test that cli will error if output path is not an existing directory.
    #[test]
    fn test_parse_output_path() {
        let result = Cli::parse_output_path("/does/not/exist/");
        assert!(result.is_err(), "Expected an error, found: {:?}", result);

        // Not a directory.
        let result = Cli::parse_output_path("tests/data/gtf_paths_missing_gtf.txt");
        assert!(result.is_err(), "Expected an error, found: {:?}", result);

        let result = Cli::parse_output_path("tests/data/");
        assert!(result.is_ok(), "{:?}", result);
    }
}
