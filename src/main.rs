use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufWriter, Read},
    path::{Path, PathBuf},
};

use axis_transform::AxisTransform;
use clap::Parser;
use lib_tsalign::{a_star_aligner::alignment_result::AlignmentStatistics, costs::U64Cost};
use log::{LevelFilter, debug, info, warn};
use noisy_float::types::R64;
use plotters::prelude::*;
use runtime_memory_csv::output_runtime_memory_csv;
use statistics_file::{
    AlignmentParameters, MergedStatisticsFile, StatisticsFile,
    alignment_strategies::AlignmentStrategyStringifyer,
};

mod axis_transform;
mod runtime_memory_csv;
mod statistics_file;

#[derive(Parser)]
struct Cli {
    /// The directory into which the plots are written.
    #[arg(long, short = 'o')]
    output_directory: PathBuf,

    /// Output the runtime and memory statistics into a CSV file.
    #[arg(long)]
    runtime_memory_csv: Option<PathBuf>,

    /// Bucket the experiments by their key (`x`-value).
    #[arg(long)]
    key_bucket_amount: Option<usize>,

    /// Make the `y`-axis an n-th-root paxislot with `n = value_polynomial_degree`.
    #[arg(long, default_value = "1.0")]
    value_polynomial_degree: f64,

    /// The statistics toml files to use for the plots.
    #[arg()]
    statistics_files: Vec<PathBuf>,

    /// Set the log level.
    #[arg(long, default_value = "info")]
    log_level: LevelFilter,

    /// Compute tsalign statistics.
    #[arg(long)]
    tsalign: bool,
}

fn main() {
    let cli = Cli::parse();

    simplelog::TermLogger::init(
        cli.log_level,
        Default::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();

    if cli.statistics_files.is_empty() {
        panic!("No statistics files given.");
    }
    if cli.key_bucket_amount == Some(0) {
        panic!("If set, key buckets must be at least one.");
    }
    if cli.value_polynomial_degree < 1.0 || R64::try_new(cli.value_polynomial_degree).is_none() {
        panic!("If set, the value polynomial degree must be at least one.");
    }

    let mut buffer = String::new();
    let statistics_files: Vec<_> = cli
        .statistics_files
        .into_iter()
        .map(|path| {
            let mut file = File::open(&path).unwrap();
            buffer.clear();
            file.read_to_string(&mut buffer).unwrap();
            toml::from_str::<StatisticsFile>(&buffer)
                .unwrap_or_else(|error| panic!("Error parsing toml file {path:?}: {error}"))
                .deserialisation_post_processing()
        })
        .collect();

    let all_statistics_files_amount = statistics_files.len();
    let alignment_strategy_stringifier =
        AlignmentStrategyStringifyer::from_statistics_files(&statistics_files);

    if let Some(runtime_memory_csv) = cli.runtime_memory_csv.as_ref() {
        info!("Creating runtime memory CSV at {runtime_memory_csv:?}");
        let runtime_memory_csv = BufWriter::new(File::create(runtime_memory_csv).unwrap());
        output_runtime_memory_csv(&statistics_files, runtime_memory_csv);
    }

    if cli.tsalign {
        info!("Reporting tsalign");
        let statistics_files: Vec<_> = statistics_files
            .iter()
            .filter(|file| {
                file.parameters.aligner == "tsalign" && file.parameters.rq_range == "all"
            })
            .cloned()
            .collect();
        if all_statistics_files_amount != statistics_files.len() {
            warn!("Dropping some statistics files for tsalign report.");
        }

        grouped_linear_bar_plot(
            &cli.output_directory,
            "opened_nodes_by_cost",
            "Alignment Cost",
            "Opened Nodes",
            (400, 400),
            cli.key_bucket_amount,
            AxisTransform::PolynomialRoot {
                degree: cli.value_polynomial_degree,
            },
            &statistics_files,
            |parameters| parameters.cost as f64,
            |file| {
                format!(
                    "{} len {}{}",
                    file.parameters.test_sequence_name,
                    file.parameters.length,
                    alignment_strategy_stringifier.stringify(file),
                )
            },
            |file| {
                let mut parameters = file.parameters.clone();
                parameters.seed = 0;
                parameters.cost = 0;
                parameters
            },
            |statistics| statistics.opened_nodes.raw(),
        );
    }
}

#[expect(clippy::too_many_arguments)]
fn grouped_linear_bar_plot<GroupName: Ord + ToString>(
    output_directory: impl AsRef<Path>,
    name: impl ToString,
    key_name: impl ToString,
    value_name: impl ToString,
    size: (u32, u32),
    key_bucket_amount: Option<usize>,
    value_transform: AxisTransform,
    statistics_files: &[StatisticsFile],
    key_fn: impl Fn(&AlignmentParameters) -> f64,
    group_name_fn: impl Fn(&StatisticsFile) -> GroupName,
    merge_key_fn: impl Fn(&StatisticsFile) -> AlignmentParameters,
    value_fn: impl Fn(&AlignmentStatistics<U64Cost>) -> f64,
) {
    let groups = group_files(statistics_files, group_name_fn);
    let (groups, min_key, max_key) =
        merge_and_sort_files_in_groups(groups, key_bucket_amount, &key_fn, merge_key_fn);

    let (min_value, max_value) = groups
        .values()
        .flat_map(|group| group.iter())
        .map(|file| value_fn(&file.max_statistics))
        .fold((f64::MAX, 0.0), |(min, max), value| {
            let min = if min > value { value } else { min };
            let max = if max < value { value } else { max };
            (min, max)
        });
    let value_epsilon = min_value
        .abs()
        .max(max_value.abs())
        .max(max_value - min_value)
        * 1e-12;
    let min_chart_value = value_transform.apply(min_value);
    let max_chart_value = value_transform.apply(max_value);

    let mut output_file_name = name.to_string();
    output_file_name.push_str(".svg");
    let mut output_file = output_directory.as_ref().to_owned();
    output_file.push(output_file_name);
    info!("Creating drawing area");
    let root: DrawingArea<SVGBackend<'_>, plotters::coord::Shift> =
        SVGBackend::new(&output_file, size).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let (min_key, max_key) = if min_key == max_key {
        debug!("All keys are the same.");
        let min_key = min_key - 0.5;
        let max_key = max_key + 0.5;
        (min_key, max_key)
    } else {
        (min_key, max_key)
    };

    info!(
        "Creating chart context with key range {min_key}..{max_key} and value range {min_chart_value}..{max_chart_value}"
    );

    let key_range_len = max_key - min_key;
    let key_margin = key_range_len / 20.0;
    let chart_value_range_len = max_chart_value - min_chart_value;
    let chart_value_margin = chart_value_range_len / 20.0;

    let mut chart = ChartBuilder::on(&root)
        .caption(name.to_string(), ("sans-serif", 24).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(50)
        .build_cartesian_2d(
            min_key - key_margin..max_key + key_margin,
            (min_chart_value - chart_value_margin) as f32
                ..(max_chart_value + chart_value_margin) as f32,
        )
        .unwrap();

    info!("Configuring chart mesh");
    chart
        .configure_mesh()
        .disable_x_mesh()
        .x_labels(groups.len())
        .x_label_formatter(&format_value)
        .y_label_formatter(&|value| format_value(&value_transform.apply_inverse(*value as f64)))
        .x_desc(key_name.to_string())
        .y_desc(format!("{} [{}]", value_name.to_string(), value_transform))
        .draw()
        .unwrap();

    let key_range = key_bucket_amount
        .map(|key_bucket_amount| key_range_len / key_bucket_amount as f64)
        .unwrap_or(1.0);
    for (group_index, ((group_name, group), style)) in groups
        .iter()
        .zip([&RED, &GREEN, &BLUE, &MAGENTA, &CYAN, &RGBColor(10, 100, 10)])
        .enumerate()
    {
        if group.is_empty() {
            info!(
                "Skipping group {} because it is empty",
                group_name.to_string()
            );
            continue;
        }

        info!("Drawing group {}", group_name.to_string());
        let coordinate_iterator = group.iter().map(|file| file.key.raw()).zip(group.iter());
        let key_shift = (((group_index as f64 + 0.5) / groups.len() as f64) * key_range * 0.7)
            - key_range * 0.5 * 0.7;

        chart
            .draw_series(coordinate_iterator.filter_map(|(key, file)| {
                let values: Vec<_> = file.contained_statistics.iter().map(&value_fn).collect();

                if values.is_empty() {
                    return None;
                }

                let quartiles = Quartiles::new(&values);
                let quartiles = Quartiles::new(&quartiles.values().map(|value| {
                    if (value as f64) < value_epsilon {
                        0.0
                    } else {
                        value_transform.apply(value as f64)
                    }
                }));
                debug!("Drawing boxplot at x = {}", key + key_shift);
                Some(Boxplot::new_vertical(key + key_shift, &quartiles).style(style))
            }))
            .unwrap()
            .label(group_name.to_string())
            .legend(move |(x, y)| Rectangle::new([(x - 5, y - 5), (x + 5, y + 5)], style));
    }

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .position(SeriesLabelPosition::LowerRight)
        .draw()
        .unwrap();
}

fn group_files<GroupName: Ord + ToString>(
    statistics_files: &[StatisticsFile],
    group_name_fn: impl Fn(&StatisticsFile) -> GroupName,
) -> BTreeMap<GroupName, Vec<StatisticsFile>> {
    info!("Grouping files");

    let mut groups: BTreeMap<_, Vec<_>> = Default::default();

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
        "groups are not of equal size:\n{}",
        {
            use std::fmt::Write;
            let mut result = String::new();
            for (group_name, group) in groups.iter() {
                writeln!(result, "{}: {}", group_name.to_string(), group.len()).unwrap();
            }
            result
        }
    );

    info!(
        "Created {} groups with {} elements each",
        groups.len(),
        groups.values().next().unwrap().len()
    );

    groups
}

fn merge_and_sort_files_in_groups<GroupName: Ord>(
    groups: BTreeMap<GroupName, Vec<StatisticsFile>>,
    key_bucket_amount: Option<usize>,
    key_fn: impl Fn(&AlignmentParameters) -> f64,
    merge_key_fn: impl Fn(&StatisticsFile) -> AlignmentParameters,
) -> (BTreeMap<GroupName, Vec<MergedStatisticsFile>>, f64, f64) {
    info!("Merge files in groups");

    let (min_key, max_key) = groups
        .values()
        .flat_map(|group| group.iter())
        .map(|file| key_fn(&file.parameters))
        .fold((f64::INFINITY, 0.0), |(min, max), value| {
            let min = if min > value { value } else { min };
            let max = if max < value { value } else { max };
            (min, max)
        });

    let mut merged_groups: BTreeMap<_, Vec<MergedStatisticsFile>> = Default::default();

    for (group_name, group) in groups {
        assert!(!group.is_empty());
        let mut merged_group: BTreeMap<_, Vec<_>> = Default::default();

        for file in group {
            let bucket_index = key_bucket_amount.map(|key_bucket_amount| {
                let key = key_fn(&file.parameters);
                let bucket_index = (key - min_key) * key_bucket_amount as f64 / (max_key - min_key);
                (bucket_index.floor() as usize)
                    .max(0)
                    .min(key_bucket_amount - 1)
            });

            let merge_key = (merge_key_fn(&file), bucket_index);
            if let Some(statistics) = merged_group.get_mut(&merge_key) {
                statistics.push(file);
            } else {
                merged_group.insert(merge_key, vec![file]);
            }
        }

        merged_groups.insert(
            group_name,
            merged_group
                .into_iter()
                .map(|((parameters, bucket_index), merge_files)| {
                    assert!(!merge_files.is_empty());
                    let key = bucket_index
                        .map(|bucket_index| {
                            ((bucket_index as f64 + 0.5) / key_bucket_amount.unwrap() as f64
                                * (max_key - min_key))
                                + min_key
                        })
                        .unwrap_or(key_fn(&parameters));
                    MergedStatisticsFile::from_statistics_files(R64::new(key), merge_files)
                })
                .collect(),
        );
    }

    let groups = sort_groups(merged_groups, |file| file.key);

    (groups, min_key, max_key)
}

fn sort_groups<GroupName: Ord, SortKey: Ord, StatisticsType>(
    mut groups: BTreeMap<GroupName, Vec<StatisticsType>>,
    sort_key_fn: impl Fn(&StatisticsType) -> SortKey,
) -> BTreeMap<GroupName, Vec<StatisticsType>> {
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
        format!("{value:.0}")
    } else if value < 1e4 {
        format!("{:.2}k", value / 1e3)
    } else if value < 1e5 {
        format!("{:.1}k", value / 1e3)
    } else if value < 1e6 {
        format!("{:.0}k", value / 1e3)
    } else if value < 1e7 {
        format!("{:.2}M", value / 1e6)
    } else if value < 1e8 {
        format!("{:.1}M", value / 1e6)
    } else if value < 1e9 {
        format!("{:.0}M", value / 1e6)
    } else if value < 1e10 {
        format!("{:.2}G", value / 1e9)
    } else if value < 1e11 {
        format!("{:.1}G", value / 1e9)
    } else if value < 1e12 {
        format!("{:.0}G", value / 1e9)
    } else {
        todo!("Support larger values");
    }
}
