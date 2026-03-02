use std::io::Write;

use crate::statistics_file::{StatisticsFile, alignment_strategies::AlignmentStrategyName};

pub fn output_runtime_memory_csv<'input>(
    statistics_files: impl IntoIterator<Item = &'input StatisticsFile>,
    mut output: impl Write,
) {
    #[expect(clippy::type_complexity)]
    let columns: &[(_, Box<dyn Fn(&StatisticsFile) -> String>)] = &[
        (
            "aligner",
            Box::new(|statistics_file| statistics_file.parameters.aligner.clone()),
        ),
        (
            "alignment_method",
            Box::new(|statistics_file| statistics_file.parameters.alignment_method.clone()),
        ),
        (
            "seed",
            Box::new(|statistics_file| format!("{}", statistics_file.parameters.seed)),
        ),
        (
            "ts_node_ord_strategy",
            Box::new(|statistics_file| {
                statistics_file.parameters.strategies[AlignmentStrategyName::NodeOrd].clone()
            }),
        ),
        (
            "ts_min_length_strategy",
            Box::new(|statistics_file| {
                statistics_file.parameters.strategies[AlignmentStrategyName::TsMinLength].clone()
            }),
        ),
        (
            "ts_total_length_strategy",
            Box::new(|statistics_file| {
                statistics_file.parameters.strategies[AlignmentStrategyName::TsTotalLength].clone()
            }),
        ),
        (
            "k",
            Box::new(|statistics_file| {
                statistics_file.parameters.strategies[AlignmentStrategyName::K].clone()
            }),
        ),
        (
            "max_chaining_successors",
            Box::new(|statistics_file| {
                statistics_file.parameters.strategies[AlignmentStrategyName::MaxChainingSuccessors]
                    .clone()
            }),
        ),
        (
            "max_exact_cost_function_cost",
            Box::new(|statistics_file| {
                statistics_file.parameters.strategies
                    [AlignmentStrategyName::MaxExactCostFunctionCost]
                    .clone()
            }),
        ),
        (
            "chaining_closed_list",
            Box::new(|statistics_file| {
                statistics_file.parameters.strategies[AlignmentStrategyName::ChainingClosedList]
                    .clone()
            }),
        ),
        (
            "chaining_open_list",
            Box::new(|statistics_file| {
                statistics_file.parameters.strategies[AlignmentStrategyName::ChainingOpenList]
                    .clone()
            }),
        ),
        (
            "rq_range",
            Box::new(|statistics_file| statistics_file.parameters.rq_range.clone()),
        ),
        (
            "cost_limit",
            Box::new(|statistics_file| statistics_file.parameters.cost_limit.clone()),
        ),
        (
            "memory_limit",
            Box::new(|statistics_file| statistics_file.parameters.memory_limit.clone()),
        ),
        (
            "runtime_seconds",
            Box::new(|statistics_file| {
                format!("{}", statistics_file.statistics.statistics().runtime)
            }),
        ),
        (
            "memory_bytes",
            Box::new(|statistics_file| {
                format!("{}", statistics_file.statistics.statistics().memory)
            }),
        ),
        (
            "ts_amount",
            Box::new(|statistics_file| {
                format!(
                    "{:.0}",
                    statistics_file
                        .statistics
                        .statistics()
                        .template_switch_amount
                )
            }),
        ),
    ];

    // Write header.
    let mut once = false;
    for (column_name, _) in columns {
        if once {
            write!(output, ",").unwrap();
        } else {
            once = true;
        }

        write!(output, "{column_name}").unwrap();
    }
    writeln!(output).unwrap();

    // Write body.
    for statistics_file in statistics_files {
        once = false;
        for (_, column) in columns {
            if once {
                write!(output, ",").unwrap();
            } else {
                once = true;
            }

            write!(output, "{}", column(statistics_file)).unwrap();
        }
        writeln!(output).unwrap();
    }
}
