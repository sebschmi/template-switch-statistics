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
            Box::new(|statistics_file: &StatisticsFile| statistics_file.parameters.aligner.clone()),
        ),
        (
            "runtime_seconds",
            Box::new(|statistics_file: &StatisticsFile| {
                format!("{}", statistics_file.statistics.statistics().runtime)
            }),
        ),
        (
            "memory_bytes",
            Box::new(|statistics_file| {
                format!("{}", statistics_file.statistics.statistics().memory)
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
