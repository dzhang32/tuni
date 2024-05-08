use clap::Parser;
use std::process;

use tuni::Cli;

fn main() {
    let cli = Cli::parse();

    match tuni::run(cli) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error executing command: {e}");
            process::exit(1);
        }
    }
}
