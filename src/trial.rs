use crate::metric::{Direction, Metric};
use crate::param::{ParamSpec, ParamValue};
use crate::{Error, Result};
use std::collections::BTreeMap;
use std::num::NonZeroU64;
use std::time::{Duration, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug)]
pub struct Trial {
    pub id: Uuid,
    pub timestamp: Duration,
    pub param_specs: BTreeMap<String, ParamSpec>,
    pub param_values: BTreeMap<String, ParamValue>,
    pub metrics: BTreeMap<String, BTreeMap<NonZeroU64, f64>>,
}

impl Trial {
    pub fn new() -> Result<Self> {
        Ok(Self {
            id: Uuid::new_v4(),
            timestamp: track!(UNIX_EPOCH.elapsed().map_err(Error::from))?,
            param_specs: BTreeMap::new(),
            param_values: BTreeMap::new(),
            metrics: BTreeMap::new(),
        })
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
