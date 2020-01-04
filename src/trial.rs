use crate::metric::{Direction, Metric};
use crate::param::{ParamSpec, ParamValue};
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::num::NonZeroU64;
use std::time::{Duration, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Trial {
    pub id: Uuid,
    pub timestamp: Duration,
    pub param_specs: BTreeMap<String, ParamSpec>,
    pub param_values: BTreeMap<String, ParamValue>,
    pub metrics: BTreeMap<String, BTreeMap<NonZeroU64, f64>>,
}

impl Trial {
    pub fn dominates(&self, other: &Self) -> bool {
        let a: HashSet<&str> = self.metrics.keys().map(|x| x.as_str()).collect();
        let b: HashSet<&str> = other.metrics.keys().map(|x| x.as_str()).collect();
        if !a.is_superset(&b) {
            return false;
        }

        let intersection: HashSet<&str> = a.intersection(&b).copied().collect();
        self.last_metrics()
            .filter(|x| intersection.contains(x.0))
            .zip(other.last_metrics().filter(|x| intersection.contains(x.0)))
            .all(|(a, b)| (a.1 > b.1) || (a.1 == b.1 && a.2 <= b.2))
    }

    fn last_metrics(&self) -> impl Iterator<Item = (&str, NonZeroU64, f64)> {
        self.metrics.iter().filter_map(|(name, reports)| {
            reports
                .iter()
                .last()
                .map(|(&step, &value)| (name.as_str(), step, value))
        })
    }

    pub fn with_id_and_timestamp(id: Uuid, timestamp: Duration) -> Self {
        Self {
            id,
            timestamp,
            param_specs: BTreeMap::new(),
            param_values: BTreeMap::new(),
            metrics: BTreeMap::new(),
        }
    }

    pub fn new() -> Result<Self> {
        Ok(Self::with_id_and_timestamp(
            Uuid::new_v4(),
            track!(UNIX_EPOCH.elapsed().map_err(Error::from))?,
        ))
    }

    pub fn report(&mut self, step: Option<NonZeroU64>, metric: &Metric) {
        let value = if metric.direction == Direction::Minimize {
            metric.value
        } else {
            -metric.value
        };

        let steps = self.metrics.entry(metric.name.clone()).or_default();
        if let Some(step) = step {
            steps.insert(step, value);
        } else if let Some(last_step) = steps.keys().rev().copied().nth(0) {
            let step = unsafe { NonZeroU64::new_unchecked(last_step.get() + 1) };
            steps.insert(step, value);
        } else {
            steps.insert(unsafe { NonZeroU64::new_unchecked(1) }, value);
        }
    }

    pub fn last_step(&self, metric_name: &str) -> Option<NonZeroU64> {
        self.metrics
            .get(metric_name)
            .and_then(|steps| steps.keys().rev().copied().nth(0))
    }
}
