use lib_tsalign::a_star_aligner::{
    alignment_result::AlignmentResult, template_switch_distance::AlignmentType,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StatisticsFile {
    #[serde(flatten)]
    statistics: AlignmentResult<AlignmentType>,

    test_sequence_name: String,
    length: usize,
    seed: u64,
    alignment_config: String,
    ts_node_ord_strategy: String,
}
