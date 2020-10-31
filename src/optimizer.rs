use self::domain::{ObjectiveSpace, ObjectiveValue, ParamNo, ParamValue, SearchSpace};
use crate::trial::{RunId, Trial};
use std::collections::BTreeMap;

pub mod domain;
pub mod optimizers;

#[derive(Debug, Clone)]
pub struct Observation {
    pub id: usize,
    pub trial: Trial,
    pub fixed_params: BTreeMap<ParamNo, ParamValue>,
    pub metrics: BTreeMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct Run {
    pub id: RunId,
    pub trial: Trial,
    pub fixed_params: BTreeMap<ParamNo, ParamValue>,
}

impl Run {
    pub fn new(id: RunId, trial: Trial) -> Self {
        Self {
            id,
            trial,
            fixed_params: BTreeMap::new(),
        }
    }
}

pub trait Optimizer {
    fn update_search_space(&mut self, search_space: &SearchSpace) -> anyhow::Result<()>;

    fn update_objective_space(&mut self, objective_space: &ObjectiveSpace) -> anyhow::Result<()>;

    fn ask(&mut self, run: &Run, param_no: ParamNo) -> anyhow::Result<ParamValue>;

    fn tell(&mut self, run: &Run, values: Option<&[ObjectiveValue]>) -> anyhow::Result<()>;

    fn resume(&mut self, run_id: RunId) -> Option<Trial>;

    // TODO(?): is_terminated or start_trial() -> New | Resume | Terminated
}

// TODO: queued-optimizer
