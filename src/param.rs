use crate::types::{FiniteF64, InclusiveRange, NonEmptyVec, NonNegF64};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ParamName(String);

impl ParamName {
    pub const fn new(name: String) -> Self {
        Self(name)
    }

    pub fn get(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamInstance {
    pub ty: ParamType,
    pub value: ParamValue,
}

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
        step: Option<NonNegF64>,
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

// TODO: delete
// #[derive(Debug, Clone, Copy)]
// pub enum OptimParamType {
//     Continuous { size: f64 },
//     Discrete { size: usize },
//     Categorical { size: usize },
//     Fidelity,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
// pub struct OptimParamValue(FiniteF64);

// impl OptimParamValue {
//     pub fn new(value: f64) -> anyhow::Result<Self> {
//         Ok(Self(FiniteF64::new(value)?))
//     }

//     pub const fn get(self) -> f64 {
//         self.0.get()
//     }
// }
