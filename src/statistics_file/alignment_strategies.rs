use std::{
    collections::{HashMap, HashSet},
    fmt::{Display, Write},
    hash::Hash,
    ops::Index,
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
    #[serde(default)]
    ts_total_length_strategy: String,
    #[serde(default)]
    k: String,
    #[serde(default)]
    max_chaining_successors: String,
    #[serde(default)]
    max_exact_cost_function_cost: String,
    #[serde(default)]
    chaining_closed_list: String,
    #[serde(default)]
    chaining_open_list: String,
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
    TsTotalLength,
    K,
    MaxChainingSuccessors,
    MaxExactCostFunctionCost,
    ChainingClosedList,
    ChainingOpenList,
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
            ts_total_length_strategy: value.map.get(&TsTotalLength).cloned().unwrap(),
            k: value.map.get(&K).cloned().unwrap(),
            max_chaining_successors: value.map.get(&MaxChainingSuccessors).cloned().unwrap(),
            max_exact_cost_function_cost: value
                .map
                .get(&MaxExactCostFunctionCost)
                .cloned()
                .unwrap(),
            chaining_closed_list: value.map.get(&ChainingClosedList).cloned().unwrap(),
            chaining_open_list: value.map.get(&ChainingOpenList).cloned().unwrap(),
        }
    }
}

impl From<AlignmentStrategiesSerde> for AlignmentStrategies {
    fn from(value: AlignmentStrategiesSerde) -> Self {
        use AlignmentStrategyName::*;
        let AlignmentStrategiesSerde {
            ts_node_ord_strategy,
            ts_min_length_strategy,
            ts_total_length_strategy,
            k,
            max_chaining_successors,
            max_exact_cost_function_cost,
            chaining_closed_list,
            chaining_open_list,
        } = value;
        Self {
            map: [
                (NodeOrd, ts_node_ord_strategy),
                (TsMinLength, ts_min_length_strategy),
                (TsTotalLength, ts_total_length_strategy),
                (K, k),
                (MaxChainingSuccessors, max_chaining_successors),
                (MaxExactCostFunctionCost, max_exact_cost_function_cost),
                (ChainingClosedList, chaining_closed_list),
                (ChainingOpenList, chaining_open_list),
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
            AlignmentStrategyName::TsTotalLength => write!(f, "ts_total_len"),
            AlignmentStrategyName::K => write!(f, "k"),
            AlignmentStrategyName::MaxChainingSuccessors => write!(f, "max_chaining_successors"),
            AlignmentStrategyName::MaxExactCostFunctionCost => {
                write!(f, "max_exact_cost_function_cost")
            }
            AlignmentStrategyName::ChainingClosedList => write!(f, "chaining_closed_list"),
            AlignmentStrategyName::ChainingOpenList => write!(f, "chaining_open_list"),
        }
    }
}

impl Index<AlignmentStrategyName> for AlignmentStrategies {
    type Output = String;

    fn index(&self, index: AlignmentStrategyName) -> &Self::Output {
        self.map.get(&index).unwrap()
    }
}
