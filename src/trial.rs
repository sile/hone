use crate::metric::{MetricInstance, MetricName, MetricValue};
use crate::param::{ParamInstance, ParamName, ParamValue};
use std::collections::BTreeMap;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct TrialId(u64);

impl TrialId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub fn fetch_and_increment(&mut self) -> Self {
        let id = Self(self.0);
        self.0 += 1;
        id
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct ObservationId(u64);

impl ObservationId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub fn fetch_and_increment(&mut self) -> Self {
        let id = Self(self.0);
        self.0 += 1;
        id
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Observation {
    #[serde(rename = "obs_id")]
    pub id: ObservationId,
    pub trial_id: TrialId,
    pub params: BTreeMap<ParamName, ParamInstance>,
    pub metrics: BTreeMap<MetricName, MetricInstance>,
    pub exit_status: Option<i32>,
}

impl Observation {
    pub fn new(obs_id: ObservationId, trial_id: TrialId) -> Self {
        Self {
            id: obs_id,
            trial_id,
            params: BTreeMap::new(),
            metrics: BTreeMap::new(),
            exit_status: None,
        }
    }

    pub fn is_max_fidelity(&self) -> bool {
        self.exit_status == Some(0)
            && self
                .params
                .values()
                .all(|p| p.is_max_fidelity().unwrap_or(true))
    }

    pub fn to_compact(&self) -> CompactObservation {
        CompactObservation {
            id: self.id,
            trial_id: self.trial_id,
            params: self
                .params
                .iter()
                .map(|(k, v)| (k.clone(), v.value.clone()))
                .collect(),
            metrics: self
                .metrics
                .iter()
                .map(|(k, v)| (k.clone(), v.value.clone()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompactObservation {
    #[serde(rename = "obs_id")]
    pub id: ObservationId,
    pub trial_id: TrialId,
    pub params: BTreeMap<ParamName, ParamValue>,
    pub metrics: BTreeMap<MetricName, MetricValue>,
}
