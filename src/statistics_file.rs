use lib_tsalign::a_star_aligner::{
    alignment_result::AlignmentResult, template_switch_distance::AlignmentType,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsFile {
    #[serde(flatten)]
    pub statistics: AlignmentResult<AlignmentType>,

    pub test_sequence_name: String,
    pub length: usize,
    pub seed: u64,
    pub alignment_config: String,
    pub ts_node_ord_strategy: String,
}
