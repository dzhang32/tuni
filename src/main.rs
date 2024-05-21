use clap::Parser;
use std::process;

use tuni::cli::Cli;

fn main() {
    let cli = Cli::parse();

    match tuni::run(cli) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}
