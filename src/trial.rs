use crate::metric::{MetricInstance, MetricName};
use crate::param::{ParamInstance, ParamName};
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

#[derive(Debug, Clone)]
pub struct Observation {
    pub id: ObservationId,
    pub trial_id: TrialId,
    pub params: BTreeMap<ParamName, ParamInstance>,
    pub metrics: BTreeMap<MetricName, MetricInstance>,
    // TODO: exit_status
}

impl Observation {
    pub fn new(obs_id: ObservationId, trial_id: TrialId) -> Self {
        Self {
            id: obs_id,
            trial_id,
            params: BTreeMap::new(),
            metrics: BTreeMap::new(),
        }
    }
}
