mod cli;
mod error;
mod gtf;
mod unify;

use clap::Parser;
use log::{info, LevelFilter};
use std::error::Error;
use std::process;

use cli::Cli;
use unify::TranscriptUnifier;

fn main() {
    let cli = Cli::parse();

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

fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    let mut transcript_unifier = TranscriptUnifier::new();
    // Due to <https://github.com/clap-rs/clap/issues/4808>, we cannot directly
    // use this as a value_parser.
    let gtf_paths = Cli::parse_gtf_paths(cli.gtf_paths)?;

    info!("Reading GTFs");

    for gtf_path in &gtf_paths {
        let gtf_file_name = gtf::extract_file_name(gtf_path);
        let mut gtf_transcripts = gtf::read_gtf(gtf_path)?;
        transcript_unifier.add_transcripts(gtf_file_name, &mut gtf_transcripts);
    }

    info!("Unifying transcripts");

    transcript_unifier.unify_transcripts();

    info!("Writing unified transcripts");

    for gtf_path in &gtf_paths {
        gtf::write_unified_gtf(gtf_path, &cli.output_dir, &transcript_unifier)?
    }

    info!("Done");

    Ok(())
}
