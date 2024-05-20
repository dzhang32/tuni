use clap::Parser;
use std::{fs, fs::File, path::PathBuf};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// A text file containing GTF paths.
    #[arg(
        short, 
        long, 
        value_name = "*.txt", 
        required = true,
    )]
    pub gtf_paths: PathBuf,

    /// Directory to store the outputted GTFs with unified transcripts.
    #[arg(
        short, 
        long, 
        value_name = "/output/dir/", 
        required = true, 
        value_parser = Cli::parse_output_dir
    )]
    pub output_dir: PathBuf,
}

impl Cli {
    /// Parse file containing GTFs, checking that GTFs exist and are readable.
    /// https://github.com/clap-rs/clap/issues/4808
    pub fn parse_gtf_paths(gtf_paths: PathBuf) -> Result<Vec<PathBuf>, String> {
        let gtf_paths = match fs::read_to_string(&gtf_paths) {
            Ok(file) => file.lines().map(PathBuf::from).collect::<Vec<PathBuf>>(),
            Err(e) => return Err(format!("{}: {}", gtf_paths.display(), e)),
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
    pub fn parse_output_dir(s: &str) -> Result<PathBuf, String> {
        let output_dir = PathBuf::from(s);
        if !output_dir.is_dir() {
            return Err(format!(
                "output_dir must point to an existing directory: {}",
                output_dir.display()
            ));
        };
        Ok(output_dir)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    /// Test that cli will error if file containing GTF paths and GTFs 1. don't
    /// exist or 2. are not readable.
    #[test]
    fn test_parse_gtf_paths() {
        let result = Cli::parse_gtf_paths(PathBuf::from("does_not_exist.txt"));
        assert!(result.is_err(), "Expected an error, found: {:?}", result);

        let result =
            Cli::parse_gtf_paths(PathBuf::from("tests/data/unit/gtf_paths_missing_gtf.txt"));
        assert!(result.is_err(), "Expected an error, found: {:?}", result);

        let result = Cli::parse_gtf_paths(PathBuf::from("tests/data/unit/gtf_paths.txt"));
        assert!(result.is_ok(), "{:?}", result);
    }

    /// Test that cli will error if output path is not an existing directory.
    #[test]
    fn test_parse_output_dir() {
        let result = Cli::parse_output_dir("/does/not/exist/");
        assert!(result.is_err(), "Expected an error, found: {:?}", result);

        // Not a directory.
        let result = Cli::parse_output_dir("tests/data/unit/gtf_paths_missing_gtf.txt");
        assert!(result.is_err(), "Expected an error, found: {:?}", result);

        let result = Cli::parse_output_dir("tests/data/unit/");
        assert!(result.is_ok(), "{:?}", result);
    }
}