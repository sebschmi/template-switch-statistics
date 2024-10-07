use std::{fs::File, io::Read, path::PathBuf};

use clap::Parser;
use statistics_file::StatisticsFile;

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

    let mut buffer = String::new();
    let statistics_files: Vec<_> = cli
        .statistics_files
        .into_iter()
        .map(|path| {
            let mut file = File::open(path).unwrap();
            buffer.clear();
            file.read_to_string(&mut buffer).unwrap();
            toml::from_str::<StatisticsFile>(&buffer).unwrap()
        })
        .collect();

    statistics_files
        .iter()
        .for_each(|statistics_file| println!("{statistics_file:?}"));
}
