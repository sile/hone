use crate::types::{FiniteF64, InclusiveRange, NonEmptyVec, NonNegF64};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct SearchSpace {
    params: BTreeMap<ParamName, ParamType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ParamName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum ParamType {
    Str(StrParamType),
    Num(NumParamType),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrParamType {
    Categorical { choices: NonEmptyVec<String> },
    Ordinal { choices: NonEmptyVec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NumParamType {
    Continous {
        range: InclusiveRange,
        ln: bool,
    },
    Discrete {
        range: InclusiveRange,
        step: NonNegF64,
    },
    Normal {
        mean: FiniteF64,
        stddev: NonNegF64,
    },
    Fidelity {
        range: InclusiveRange,
        step: NonNegF64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum ParamValue {
    Str(String),
    Num(FiniteF64),
}

impl std::fmt::Display for ParamValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Str(v) => write!(f, "{}", v),
            Self::Num(v) => write!(f, "{}", v.get()),
        }
    }
}
