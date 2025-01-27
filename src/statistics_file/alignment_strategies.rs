use std::{
    collections::{HashMap, HashSet},
    fmt::{Display, Write},
    hash::Hash,
};

use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use super::StatisticsFile;

#[derive(Serialize, Deserialize)]
pub struct AlignmentStrategiesSerde {
    #[serde(default)]
    ts_node_ord_strategy: String,
    #[serde(default)]
    ts_min_length_strategy: String,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(from = "AlignmentStrategiesSerde", into = "AlignmentStrategiesSerde")]
pub struct AlignmentStrategies {
    map: HashMap<AlignmentStrategyName, String>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, EnumIter)]
pub enum AlignmentStrategyName {
    NodeOrd,
    TsMinLength,
}

pub struct AlignmentStrategyStringifyer {
    relevant_strategies: Vec<AlignmentStrategyName>,
}

impl AlignmentStrategyStringifyer {
    pub fn new<'item>(
        strategy_selections: impl IntoIterator<Item = &'item AlignmentStrategies>,
    ) -> Self {
        let mut existing_strategy_values: HashMap<AlignmentStrategyName, HashSet<String>> =
            Default::default();
        for strategy_selection in strategy_selections {
            for (name, value) in &strategy_selection.map {
                if let Some(set) = existing_strategy_values.get_mut(name) {
                    set.insert(value.clone());
                } else {
                    existing_strategy_values.insert(*name, [value.clone()].into());
                }
            }
        }

        Self {
            relevant_strategies: AlignmentStrategyName::iter()
                .filter(|name| {
                    existing_strategy_values
                        .get(name)
                        .unwrap_or(&HashSet::new())
                        .len()
                        > 1
                })
                .collect(),
        }
    }

    pub fn from_statistics_files(files: &[StatisticsFile]) -> Self {
        Self::new(files.iter().map(|file| &file.parameters.strategies))
    }

    pub fn stringify(&self, file: &StatisticsFile) -> String {
        let mut result = String::new();
        for name in &self.relevant_strategies {
            write!(
                result,
                "; {name} {}",
                file.parameters.strategies.map.get(name).unwrap()
            )
            .unwrap();
        }
        result
    }
}

impl AlignmentStrategies {
    pub fn is_ari_email(&self) -> bool {
        self.map
            .get(&AlignmentStrategyName::NodeOrd)
            .map(String::as_str)
            == Some("anti-diagonal")
            && self
                .map
                .get(&AlignmentStrategyName::TsMinLength)
                .map(String::as_str)
                == Some("lookahead")
    }
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
            .into(),
        }
    }
}

impl Display for AlignmentStrategyName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlignmentStrategyName::NodeOrd => write!(f, "node_ord"),
            AlignmentStrategyName::TsMinLength => write!(f, "ts_min_len"),
        }
    }
}
