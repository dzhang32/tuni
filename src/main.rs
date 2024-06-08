mod cli;
mod error;
mod gtf_gff;
mod unify;

use clap::Parser;
use log::{info, LevelFilter};
use std::error::Error;
use std::process;

use cli::Cli;
use unify::TranscriptUnifier;

/// Responsible for parsing cli arguments, setting the log level and
/// printing errors.
fn main() {
    let cli = Cli::parse();

    // By default, warn users.
    // Warning indicates potentially incorrectly formatted input.
    let log_level = match cli.verbose {
        true => LevelFilter::Info,
        false => LevelFilter::Warn,
    };
    env_logger::Builder::new().filter_level(log_level).init();

    match run(cli) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

/// Executes tuni, prints top-level logs and returns unrecoverable errors.
fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    let mut transcript_unifier = TranscriptUnifier::new();
    // Due to <https://github.com/clap-rs/clap/issues/4808>, value_parser cannot
    // directly use this function.
    let (gtf_gff_extension, gtf_gff_paths) = Cli::parse_gtf_gff_paths(cli.gtf_gff_path)?;

    info!("Reading GTF/GFFs");

    for gtf_gff_path in &gtf_gff_paths {
        let gtf_gff_file_name = gtf_gff::extract_file_name(gtf_gff_path);
        let mut gtf_gff_transcripts = gtf_gff::read_gtf_gff(gtf_gff_path)?;
        transcript_unifier.group_transcripts(gtf_gff_file_name, &mut gtf_gff_transcripts);
    }

    info!("Unifying transcripts");

    transcript_unifier.unify_transcripts();

    info!("Writing unified transcripts");

    for gtf_gff_path in &gtf_gff_paths {
        gtf_gff::write_unified_gtf_gff(
            &gtf_gff_extension,
            gtf_gff_path,
            &cli.output_dir,
            &transcript_unifier,
        )?
    }

    info!("Done");

    Ok(())
}
