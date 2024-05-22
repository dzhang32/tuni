mod cli;
mod error;
mod gtf;
mod unify;

use clap::Parser;
use log::info;
use std::error::Error;
use std::process;

use cli::Cli;
use unify::TranscriptUnifier;

fn main() {
    let cli = Cli::parse();

    match run(cli) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

pub fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let mut transcript_unifier = TranscriptUnifier::new();
    // https://github.com/clap-rs/clap/issues/4808.
    let gtf_paths = Cli::parse_gtf_paths(cli.gtf_paths)?;

    for gtf_path in &gtf_paths {
        // TODO: create helper function.
        let gtf_file_name = gtf::extract_file_name(gtf_path);
        let mut gtf_transcripts = gtf::read_gtf(gtf_path)?;
        transcript_unifier.add_transcripts(gtf_file_name, &mut gtf_transcripts);
    }

    transcript_unifier.unify_transcripts();

    for gtf_path in &gtf_paths {
        gtf::write_unified_gtf(gtf_path, &cli.output_dir, &transcript_unifier)?
    }

    Ok(())
}
