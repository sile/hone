use crate::types::FiniteF64;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MetricName(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MetricValue(FiniteF64);

impl MetricValue {
    pub fn new(value: f64) -> anyhow::Result<Self> {
        Ok(Self(FiniteF64::new(value)?))
    }

    pub const fn get(self) -> f64 {
        self.0.get()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricType {
    pub objective: Option<Objective>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Objective {
    Minimize,
    Maximize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricInstance {
    pub ty: MetricType,
    pub value: MetricValue,
}
