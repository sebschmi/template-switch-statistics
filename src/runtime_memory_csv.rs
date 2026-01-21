use std::io::Write;

use crate::statistics_file::StatisticsFile;

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
