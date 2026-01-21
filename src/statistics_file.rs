use alignment_strategies::AlignmentStrategies;
use lib_tsalign::{
    a_star_aligner::{
        alignment_result::{AlignmentResult, AlignmentStatistics},
        template_switch_distance::AlignmentType,
    },
    costs::U64Cost,
};
use noisy_float::types::{R64, r64};
use serde::{Deserialize, Serialize};

pub mod alignment_strategies;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct StatisticsFile {
    #[serde(default = "default_statistics")]
    pub statistics: AlignmentResult<AlignmentType, U64Cost>,

    #[serde(flatten)]
    pub parameters: AlignmentParameters,

    #[serde(default)]
    pub template_switch_amount: u64,
}

fn default_statistics() -> AlignmentResult<AlignmentType, U64Cost> {
    AlignmentResult::WithoutTarget {
        statistics: AlignmentStatistics::zero(),
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct AlignmentParameters {
    pub test_sequence_name: String,
    pub aligner: String,
    pub alignment_method: String,
    pub length: usize,
    #[serde(skip)]
    pub cost: u64,
    pub seed: u64,
    #[serde(default)]
    pub alignment_config: String,
    pub rq_range: String,
    pub cost_limit: String,
    pub memory_limit: String,

    pub runtime_raw: Vec<String>,
    /// Memory in kibibytes.
    pub memory_raw: u64,

    #[serde(flatten)]
    pub strategies: AlignmentStrategies,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct MergedStatisticsFile {
    pub min_statistics: AlignmentStatistics<U64Cost>,
    pub max_statistics: AlignmentStatistics<U64Cost>,
    pub mean_statistics: AlignmentStatistics<U64Cost>,
    pub median_statistics: AlignmentStatistics<U64Cost>,
    pub contained_statistics: Vec<AlignmentStatistics<U64Cost>>,

    pub key: R64,
}

impl StatisticsFile {
    pub fn deserialisation_post_processing(mut self) -> Self {
        self.parameters.cost = self.statistics.statistics().cost.raw() as u64;
        self.unpack_runtime();
        self.unpack_memory();
        self.fix_template_switch_amount();
        self
    }

    fn unpack_runtime(&mut self) {
        self.statistics.statistics_mut().runtime = r64(0.0);

        for runtime in &self.parameters.runtime_raw {
            let runtime = runtime.split(':').collect::<Vec<_>>();
            assert!(runtime.len() >= 2);
            assert!(runtime.len() <= 3);

            let mut factor = r64(1.0);
            for runtime in runtime.iter().rev() {
                let runtime: f64 = runtime.parse().unwrap();
                let runtime = r64(runtime) * factor;
                self.statistics.statistics_mut().runtime += runtime;
                factor *= r64(60.0);
            }
        }
    }

    fn unpack_memory(&mut self) {
        self.statistics.statistics_mut().memory =
            r64(self.parameters.memory_raw as f64) * r64(1024.0);
    }

    fn fix_template_switch_amount(&mut self) {
        if self.template_switch_amount > 0 {
            assert_eq!(self.statistics.statistics_mut().template_switch_amount, 0.0);
            self.statistics.statistics_mut().template_switch_amount =
                r64(self.template_switch_amount as f64);
        }
    }
}

impl MergedStatisticsFile {
    pub fn from_statistics_files(key: R64, statistics_files: Vec<StatisticsFile>) -> Self {
        assert!(!statistics_files.is_empty());

        let alignment_statistics = statistics_files
            .iter()
            .map(|file| file.statistics.statistics().clone())
            .collect::<Vec<_>>();

        let mut result = Self {
            min_statistics: AlignmentStatistics::max_value(),
            max_statistics: AlignmentStatistics::min_value(),
            mean_statistics: AlignmentStatistics::zero(),
            median_statistics: AlignmentStatistics::piecewise_percentile(
                &alignment_statistics,
                R64::new(0.5),
            ),
            contained_statistics: Default::default(),

            key,
        };

        for statistics in &alignment_statistics {
            result.min_statistics = result.min_statistics.piecewise_min(statistics);
            result.max_statistics = result.max_statistics.piecewise_max(statistics);
            result.mean_statistics = result.mean_statistics.piecewise_add(statistics);
            result.contained_statistics.push(statistics.clone());
        }

        result.mean_statistics = result
            .mean_statistics
            .piecewise_div(R64::new(alignment_statistics.len() as f64));

        result
    }
}
