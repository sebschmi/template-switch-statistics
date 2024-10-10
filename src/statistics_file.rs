use lib_tsalign::a_star_aligner::{
    alignment_result::{AlignmentResult, AlignmentStatistics},
    template_switch_distance::AlignmentType,
};
use noisy_float::types::R64;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct StatisticsFile {
    #[serde(flatten)]
    pub statistics: AlignmentResult<AlignmentType>,

    #[serde(flatten)]
    pub parameters: AlignmentParameters,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct AlignmentParameters {
    pub test_sequence_name: String,
    pub length: usize,
    #[serde(skip)]
    pub cost: u64,
    pub seed: u64,
    pub alignment_config: String,
    pub ts_node_ord_strategy: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct MergedStatisticsFile {
    pub min_statistics: AlignmentStatistics,
    pub max_statistics: AlignmentStatistics,
    pub mean_statistics: AlignmentStatistics,
    pub median_statistics: AlignmentStatistics,
    pub contained_statistics: Vec<AlignmentStatistics>,

    pub key: R64,
}

impl StatisticsFile {
    pub fn deserialisation_post_processing(mut self) -> Self {
        self.parameters.cost = self.statistics.statistics.cost.raw() as u64;
        self
    }
}

impl MergedStatisticsFile {
    pub fn from_statistics_files(key: R64, statistics_files: Vec<StatisticsFile>) -> Self {
        assert!(!statistics_files.is_empty());

        let mut result = Self {
            min_statistics: AlignmentStatistics::max_value(),
            max_statistics: AlignmentStatistics::min_value(),
            mean_statistics: AlignmentStatistics::zero(),
            median_statistics: AlignmentStatistics::piecewise_percentile(
                &statistics_files
                    .iter()
                    .map(|file| file.statistics.statistics.clone())
                    .collect::<Vec<_>>(),
                R64::new(0.5),
            ),
            contained_statistics: Default::default(),

            key,
        };

        for statistics in &statistics_files {
            let statistics = &statistics.statistics.statistics;
            result.min_statistics = result.min_statistics.piecewise_min(statistics);
            result.max_statistics = result.max_statistics.piecewise_max(statistics);
            result.mean_statistics = result.mean_statistics.piecewise_add(statistics);
            result.contained_statistics.push(statistics.clone());
        }

        result.mean_statistics = result
            .mean_statistics
            .piecewise_div(R64::new(statistics_files.len() as f64));

        result
    }
}
