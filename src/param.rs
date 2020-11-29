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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParamInstance {
    pub ty: ParamType,
    pub value: ParamValue,
}

impl ParamInstance {
    pub const fn new(ty: ParamType, value: ParamValue) -> Self {
        Self { ty, value }
    }

    pub fn is_max_fidelity(&self) -> Option<bool> {
        if let Self {
            ty: ParamType::Num(NumParamType::Fidelity(ty)),
            value: ParamValue::Num(value),
        } = self
        {
            Some(ty.range.max() == *value)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum ParamType {
    Str(StrParamType),
    Num(NumParamType),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrParamType {
    Categorical(CategoricalParamType),
    Ordinal(OrdinalParamType),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CategoricalParamType {
    choices: NonEmptyVec<String>,
}

impl CategoricalParamType {
    pub fn new(choices: Vec<String>) -> anyhow::Result<Self> {
        Ok(Self {
            choices: NonEmptyVec::new(choices)?,
        })
    }

    pub fn choices(&self) -> &NonEmptyVec<String> {
        &self.choices
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OrdinalParamType {
    choices: NonEmptyVec<String>,
}

impl OrdinalParamType {
    pub fn new(choices: Vec<String>) -> anyhow::Result<Self> {
        Ok(Self {
            choices: NonEmptyVec::new(choices)?,
        })
    }

    pub fn choices(&self) -> &NonEmptyVec<String> {
        &self.choices
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NumParamType {
    Continous(ContinousParamType),
    Discrete(DiscreteParamType),
    Normal(NormalParamType),
    Fidelity(FidelityParamType),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")] // TOOD: try_from UncheckedContinousParamType
pub struct ContinousParamType {
    range: InclusiveRange,
    ln: bool,
}

impl ContinousParamType {
    pub fn new(min: f64, max: f64, ln: bool) -> anyhow::Result<Self> {
        let range = InclusiveRange::new(min, max)?;
        if ln {
            anyhow::ensure!(range.min().get().is_sign_positive(), "TODO");
        }
        Ok(Self { range, ln })
    }

    pub const fn range(&self) -> InclusiveRange {
        self.range
    }

    pub const fn ln(&self) -> bool {
        self.ln
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct NormalParamType {
    mean: FiniteF64,
    stddev: NonNegF64,
}

impl NormalParamType {
    pub fn new(mean: f64, stddev: f64) -> anyhow::Result<Self> {
        Ok(Self {
            mean: FiniteF64::new(mean)?,
            stddev: NonNegF64::new(stddev)?,
        })
    }

    pub const fn mean(&self) -> FiniteF64 {
        self.mean
    }

    pub const fn stddev(&self) -> NonNegF64 {
        self.stddev
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DiscreteParamType {
    range: InclusiveRange,
    step: NonNegF64,
}

impl DiscreteParamType {
    pub fn new(min: f64, max: f64, step: f64) -> anyhow::Result<Self> {
        let mut range = InclusiveRange::new(min, max)?;
        let max = range.min().get() + (range.width().get() / step).floor() * step;
        range = InclusiveRange::new(range.min().get(), max)?;
        Ok(Self {
            range,
            step: NonNegF64::new(step)?,
        })
    }

    pub const fn range(&self) -> InclusiveRange {
        self.range
    }

    pub const fn step(&self) -> NonNegF64 {
        self.step
    }

    pub fn count(&self) -> u64 {
        (self.range.width().get() / self.step.get()) as u64
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FidelityParamType {
    range: InclusiveRange,
    step: Option<NonNegF64>,
}

impl FidelityParamType {
    pub fn new(min: f64, max: f64, step: Option<f64>) -> anyhow::Result<Self> {
        let mut range = InclusiveRange::new(min, max)?;
        if let Some(step) = step {
            let max = range.min().get() + (range.width().get() / step).floor() * step;
            range = InclusiveRange::new(range.min().get(), max)?;
        }
        Ok(Self {
            range,
            step: step.map(|step| NonNegF64::new(step)).transpose()?,
        })
    }

    pub const fn range(&self) -> InclusiveRange {
        self.range
    }

    pub const fn step(&self) -> Option<NonNegF64> {
        self.step
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
