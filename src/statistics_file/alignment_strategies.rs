use std::{collections::HashMap, hash::Hash};

use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Serialize, Deserialize)]
pub struct AlignmentStrategiesSerde {
    ts_node_ord_strategy: String,
    ts_min_length_strategy: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(from = "AlignmentStrategiesSerde", into = "AlignmentStrategiesSerde")]
pub struct AlignmentStrategies {
    map: HashMap<AlignmentStrategyName, String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, EnumIter)]
pub enum AlignmentStrategyName {
    NodeOrd,
    TsMinLength,
}

impl Ord for AlignmentStrategies {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        for name in AlignmentStrategyName::iter() {
            match self.map.get(&name).cmp(&other.map.get(&name)) {
                std::cmp::Ordering::Less => return std::cmp::Ordering::Less,
                std::cmp::Ordering::Equal => { /* continue */ }
                std::cmp::Ordering::Greater => return std::cmp::Ordering::Greater,
            }
        }

        std::cmp::Ordering::Equal
    }
}

impl PartialOrd for AlignmentStrategies {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for AlignmentStrategies {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for name in AlignmentStrategyName::iter() {
            self.map.get(&name).hash(state)
        }
    }
}

impl From<AlignmentStrategies> for AlignmentStrategiesSerde {
    fn from(value: AlignmentStrategies) -> Self {
        use AlignmentStrategyName::*;
        Self {
            ts_node_ord_strategy: value.map.get(&NodeOrd).cloned().unwrap(),
            ts_min_length_strategy: value.map.get(&TsMinLength).cloned().unwrap(),
        }
    }
}

impl From<AlignmentStrategiesSerde> for AlignmentStrategies {
    fn from(value: AlignmentStrategiesSerde) -> Self {
        use AlignmentStrategyName::*;
        let AlignmentStrategiesSerde {
            ts_node_ord_strategy,
            ts_min_length_strategy,
        } = value;
        Self {
            map: [
                (NodeOrd, ts_node_ord_strategy),
                (TsMinLength, ts_min_length_strategy),
            ]
            .into_iter()
            .collect(),
        }
    }
}
