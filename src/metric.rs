use crate::types::FiniteF64;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MetricName(String);

impl MetricName {
    pub const fn new(name: String) -> Self {
        Self(name)
    }

    pub fn get(&self) -> &str {
        &self.0
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricType {
    Minimize,
    Maximize,
    Record,
    Judge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricInstance {
    pub ty: MetricType,
    pub value: MetricValue,
}

impl MetricInstance {
    pub const fn new(ty: MetricType, value: MetricValue) -> Self {
        Self { ty, value }
    }

    pub fn is_better_than(&self, other: MetricValue) -> bool {
        match self.ty {
            MetricType::Minimize => self.value < other,
            MetricType::Maximize => self.value > other,
            MetricType::Record => false,
            MetricType::Judge => false,
        }
    }
}
