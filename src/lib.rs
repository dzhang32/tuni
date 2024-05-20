// TODO: should I switch these to use?
pub mod cli;
mod gtf;
mod unify;

use cli::Cli;
use gtf::{read_gtf, write_unified_gtf};
use unify::TranscriptUnifier;

use std::{error::Error, rc::Rc};

pub fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    let mut transcript_unifier = TranscriptUnifier::new();
    // https://github.com/clap-rs/clap/issues/4808.
    let gtf_paths = Cli::parse_gtf_paths(cli.gtf_paths)?;

    for gtf_path in &gtf_paths {
        let gtf_file_name = gtf_path.file_name().unwrap().to_str().unwrap();
        let mut gtf_transcripts = read_gtf(gtf_path);
        transcript_unifier.add_transcripts(Rc::from(gtf_file_name), &mut gtf_transcripts);
    }

    transcript_unifier.unify_transcripts();

    for gtf_path in &gtf_paths {
        write_unified_gtf(gtf_path, &cli.output_dir, &transcript_unifier)
    }

    Ok(())
}
