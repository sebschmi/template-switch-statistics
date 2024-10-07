use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use clap::Parser;
use plotters::prelude::*;
use statistics_file::StatisticsFile;

mod statistics_file;

#[derive(Parser)]
struct Cli {
    /// The directory into which the plots are written.
    #[arg(long, short = 'o')]
    output_directory: PathBuf,

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

    grouped_linear_bar_plot(
        &cli.output_directory,
        "opened_nodes",
        (400, 400),
        &statistics_files,
        |file| file.length,
        |file| file.length as f64,
        |file| file.test_sequence_name.clone(),
        |file| file.statistics.opened_nodes as f64,
    );
}

#[allow(clippy::too_many_arguments)]
fn grouped_linear_bar_plot<SortKey: Ord>(
    output_directory: impl AsRef<Path>,
    name: impl ToString,
    size: (u32, u32),
    statistics_files: &[StatisticsFile],
    sort_key_fn: impl Fn(&StatisticsFile) -> SortKey,
    key_fn: impl Fn(&StatisticsFile) -> f64,
    group_fn: impl Fn(&StatisticsFile) -> String,
    value_fn: impl Fn(&StatisticsFile) -> f64,
) {
    let mut groups: HashMap<_, Vec<_>> = Default::default();

    for file in statistics_files {
        let group_name = group_fn(file);
        if let Some(group) = groups.get_mut(&group_name) {
            group.push(file.clone());
        } else {
            groups.insert(group_name, vec![file.clone()]);
        }
    }

    groups
        .values_mut()
        .for_each(|group| group.sort_unstable_by_key(&sort_key_fn));

    assert!(
        groups
            .values()
            .skip(1)
            .fold(
                (groups.values().next().unwrap().len(), true),
                |(len, truth), group| (len, truth && group.len() == len)
            )
            .1,
        "groups are not of equal size"
    );

    let mut output_file_name = name.to_string();
    output_file_name.push_str(".svg");
    let mut output_file = output_directory.as_ref().to_owned();
    output_file.push(output_file_name);

    let (min_key, max_key) =
        statistics_files
            .iter()
            .map(&key_fn)
            .fold((f64::INFINITY, 0.0), |(min, max), value| {
                (
                    if min < value { min } else { value },
                    if max > value { max } else { value },
                )
            });
    let max_value = statistics_files
        .iter()
        .map(&value_fn)
        .fold(0.0, |acc, value| if acc < value { value } else { acc });

    let root = SVGBackend::new(&output_file, size).into_drawing_area();
    root.fill(&TRANSPARENT).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption(name.to_string(), ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(min_key / 1.05..max_key * 1.05, 0.0..max_value * 1.05)
        .unwrap();

    chart
        .configure_mesh()
        .disable_x_mesh()
        .x_labels(groups.len())
        .x_label_formatter(&format_value)
        .y_label_formatter(&format_value)
        .draw()
        .unwrap();

    for (group, style) in groups
        .values()
        .zip([&RED, &GREEN, &BLUE, &MAGENTA, &CYAN, &YELLOW])
    {
        let coordinate_iterator = group.iter().map(&key_fn).zip(group.iter().map(&value_fn));

        chart
            .draw_series(LineSeries::new(coordinate_iterator.clone(), style))
            .unwrap();
        chart
            .draw_series(
                coordinate_iterator.map(|coordinate| Circle::new(coordinate, 3, style.filled())),
            )
            .unwrap();
    }
}

fn format_value(value: &f64) -> String {
    let value = *value;
    assert!(
        value.is_sign_positive() && value.is_finite() && !value.is_nan() && !value.is_subnormal(),
        "Unsupported value: {value}"
    );

    if value == 0.0 {
        "0".to_string()
    } else if value < 1e3 {
        format!("{:.0}", value)
    } else if value < 1e6 {
        format!("{:.0}k", value / 1e3)
    } else if value < 1e9 {
        format!("{:.0}M", value / 1e6)
    } else if value < 1e12 {
        format!("{:.0}G", value / 1e9)
    } else {
        todo!("Support larger values");
    }
}
