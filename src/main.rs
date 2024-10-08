use std::{
    collections::HashMap,
    fs::File,
    hash::Hash,
    io::Read,
    path::{Path, PathBuf},
};

use clap::Parser;
use lib_tsalign::a_star_aligner::alignment_result::AlignmentStatistics;
use log::info;
use plotters::prelude::*;
use statistics_file::{AlignmentParameters, MergedStatisticsFile, StatisticsFile};

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
    simplelog::TermLogger::init(
        log::LevelFilter::Info,
        Default::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();

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
        |file| file.parameters.length,
        |file| file.parameters.length as f64,
        |file| file.parameters.test_sequence_name.clone(),
        |file| {
            let mut parameters = file.parameters.clone();
            parameters.seed = 0;
            parameters
        },
        |statistics| statistics.opened_nodes.raw(),
    );
}

#[allow(clippy::too_many_arguments)]
fn grouped_linear_bar_plot<GroupName: Hash + Eq + ToString, SortKey: Ord>(
    output_directory: impl AsRef<Path>,
    name: impl ToString,
    size: (u32, u32),
    statistics_files: &[StatisticsFile],
    sort_key_fn: impl Fn(&MergedStatisticsFile) -> SortKey,
    key_fn: impl Fn(&MergedStatisticsFile) -> f64,
    group_name_fn: impl Fn(&StatisticsFile) -> GroupName,
    merge_key_fn: impl Fn(&StatisticsFile) -> AlignmentParameters,
    value_fn: impl Fn(&AlignmentStatistics) -> f64,
) {
    let groups = group_files(statistics_files, group_name_fn);
    let groups = merge_files_in_groups(groups, merge_key_fn);
    let groups = sort_groups(groups, sort_key_fn);

    let mut output_file_name = name.to_string();
    output_file_name.push_str(".svg");
    let mut output_file = output_directory.as_ref().to_owned();
    output_file.push(output_file_name);

    let (min_key, max_key) = groups
        .values()
        .flat_map(|group| group.iter())
        .map(&key_fn)
        .fold((f64::INFINITY, 0.0), |(min, max), value| {
            (
                if min < value { min } else { value },
                if max > value { max } else { value },
            )
        });
    let max_value = groups
        .values()
        .flat_map(|group| group.iter())
        .map(|file| value_fn(&file.max_statistics))
        .fold(0.0, |acc, value| if acc < value { value } else { acc });

    info!("Creating drawing area");
    let root = SVGBackend::new(&output_file, size).into_drawing_area();
    root.fill(&TRANSPARENT).unwrap();

    info!("Creating chart context with key range {min_key}..{max_key} and max value {max_value}");
    let mut chart = ChartBuilder::on(&root)
        .caption(name.to_string(), ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(min_key / 1.05..max_key * 1.05, 0.0..max_value * 1.05)
        .unwrap();

    info!("Configuring chart mesh");
    chart
        .configure_mesh()
        .disable_x_mesh()
        .x_labels(groups.len())
        .x_label_formatter(&format_value)
        .y_label_formatter(&format_value)
        .draw()
        .unwrap();

    for ((group_name, group), style) in groups
        .iter()
        .zip([&RED, &GREEN, &BLUE, &MAGENTA, &CYAN, &YELLOW])
    {
        info!("Drawing group {}", group_name.to_string());
        let coordinate_iterator = group
            .iter()
            .map(&key_fn)
            .zip(group.iter().map(|file| value_fn(&file.mean_statistics)));

        chart
            .draw_series(LineSeries::new(coordinate_iterator.clone(), style))
            .unwrap();
        chart
            .draw_series(coordinate_iterator.map(|coordinate| Circle::new(coordinate, 3, style)))
            .unwrap()
            .label(group_name.to_string())
            .legend(move |(x, y)| Rectangle::new([(x - 5, y - 5), (x + 5, y + 5)], style));
    }

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()
        .unwrap();
}

fn group_files<GroupName: Hash + Eq>(
    statistics_files: &[StatisticsFile],
    group_name_fn: impl Fn(&StatisticsFile) -> GroupName,
) -> HashMap<GroupName, Vec<StatisticsFile>> {
    info!("Grouping files");

    let mut groups: HashMap<_, Vec<_>> = Default::default();

    for file in statistics_files {
        let group_name = group_name_fn(file);
        if let Some(group) = groups.get_mut(&group_name) {
            group.push(file.clone());
        } else {
            groups.insert(group_name, vec![file.clone()]);
        }
    }

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

    info!(
        "Created {} groups with {} elements each",
        groups.len(),
        groups.values().next().unwrap().len()
    );

    groups
}

fn merge_files_in_groups<GroupName: Hash + Eq>(
    groups: HashMap<GroupName, Vec<StatisticsFile>>,
    merge_key_fn: impl Fn(&StatisticsFile) -> AlignmentParameters,
) -> HashMap<GroupName, Vec<MergedStatisticsFile>> {
    info!("Merge files in groups");

    let mut merged_groups = HashMap::new();

    for (group_name, group) in groups {
        let mut merged_group: HashMap<_, Vec<_>> = Default::default();

        for file in group {
            let merge_key = merge_key_fn(&file);
            if let Some(statistics) = merged_group.get_mut(&merge_key) {
                statistics.push(file);
            } else {
                merged_group.insert(merge_key, vec![file]);
            }
        }

        merged_groups.insert(
            group_name,
            merged_group.into_values().map(Into::into).collect(),
        );
    }

    merged_groups
}

fn sort_groups<GroupName: Hash + Eq, SortKey: Ord, StatisticsType>(
    mut groups: HashMap<GroupName, Vec<StatisticsType>>,
    sort_key_fn: impl Fn(&StatisticsType) -> SortKey,
) -> HashMap<GroupName, Vec<StatisticsType>> {
    info!("Sort groups");

    groups
        .values_mut()
        .for_each(|group| group.sort_unstable_by_key(&sort_key_fn));
    groups
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
