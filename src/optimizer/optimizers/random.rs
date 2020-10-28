use super::super::domain::{
    ObjectiveSpace, ObjectiveValue, ParamNo, ParamType, ParamValue, SearchSpace,
};
use super::super::{Optimizer, Run};
use crate::rng::ArcRng;
use crate::trial::{RunId, Trial};
use rand::Rng;

#[derive(Debug)]
pub struct RandomOptimizer {
    rng: ArcRng,
    search_space: SearchSpace,
}

impl RandomOptimizer {
    pub fn new(rng: ArcRng) -> Self {
        Self {
            rng,
            search_space: SearchSpace::new(),
        }
    }
}

impl Optimizer for RandomOptimizer {
    fn update_search_space(&mut self, search_space: &SearchSpace) -> anyhow::Result<()> {
        self.search_space = search_space.clone();
        Ok(())
    }

    fn update_objective_space(&mut self, _objective_space: &ObjectiveSpace) -> anyhow::Result<()> {
        Ok(())
    }

    fn ask(&mut self, _run: &Run, param_no: ParamNo) -> anyhow::Result<ParamValue> {
        let param_type = self.search_space.get_param_type(param_no)?;
        let v = match param_type {
            ParamType::Continuous { size } => self.rng.gen_range(0.0, size),
            ParamType::Discrete { size } => self.rng.gen_range(0, size) as f64,
            ParamType::Categorical { size } => self.rng.gen_range(0, size) as f64,
            ParamType::Fidelity => 1.0,
        };
        ParamValue::new(v)
    }

    fn tell(&mut self, _run: &Run, _values: Option<&[ObjectiveValue]>) -> anyhow::Result<()> {
        Ok(())
    }

    fn resume(&mut self, _run_id: RunId) -> Option<Trial> {
        None
    }
}
