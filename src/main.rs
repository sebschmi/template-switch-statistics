use std::path::PathBuf;

use clap::Parser;

mod statistics_file;

#[derive(Parser)]
struct Cli {
    /// The statistics toml files to use for the plots.
    #[arg()]
    statistics_files: Vec<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    if cli.statistics_files.is_empty() {
        panic!("No statistics files given.");
    }

    println!("Hello, world!");
}
