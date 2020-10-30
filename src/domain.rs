pub use crate::optimizer::domain::ObjectiveValue;

#[derive(Debug)]
pub struct SearchSpace {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ParamType {
    Str { ty: StrParamType },
    Num { ty: NumParamType, fidelity: bool },
}

impl ParamType {
    pub fn categorical(choices: Vec<String>) -> Self {
        Self::Str {
            ty: StrParamType::Categorical { choices },
        }
    }

    pub fn ordinal(choices: Vec<String>) -> Self {
        Self::Str {
            ty: StrParamType::Ordinal { choices },
        }
    }

    pub fn normal(mean: f64, stddev: f64) -> Self {
        Self::Num {
            ty: NumParamType::Normal { mean, stddev },
            fidelity: false,
        }
    }

    pub fn continous(start: f64, end: f64, ln: bool, fidelity: bool) -> Self {
        Self::Num {
            ty: NumParamType::Continous { start, end, ln },
            fidelity,
        }
    }

    pub fn discrete(start: f64, end: f64, step: f64, fidelity: bool) -> Self {
        Self::Num {
            ty: NumParamType::Discrete { start, end, step },
            fidelity,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum StrParamType {
    Categorical { choices: Vec<String> },
    Ordinal { choices: Vec<String> },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum NumParamType {
    Continous { start: f64, end: f64, ln: bool },
    Discrete { start: f64, end: f64, step: f64 },
    Normal { mean: f64, stddev: f64 },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ParamValue {
    Str(String),
    Num(f64),
}

impl std::fmt::Display for ParamValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Str(v) => write!(f, "{}", v),
            Self::Num(v) => write!(f, "{}", v),
        }
    }
}

// TODO(?): s/Objective/Metric/
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObjectiveType {
    pub minimize: bool,
    pub ignore: bool,
}
