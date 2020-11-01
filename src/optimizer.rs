use crate::param::{ParamName, ParamType, ParamValue};
use crate::trial::{Observation, TrialId};

//pub mod optimizers;

pub trait Optimizer {
    fn ask(
        &mut self,
        obs: &Observation,
        param_name: &ParamName,
        param_type: &ParamType,
    ) -> anyhow::Result<ParamValue>;

    fn tell(&mut self, obs: &Observation) -> anyhow::Result<()>;

    fn next_action(&mut self) -> anyhow::Result<Action>;
}

#[derive(Debug, Clone)]
pub enum Action {
    CreateTrial,
    ResumeTrial { trial_id: TrialId },
    FinishTrial { trial_id: TrialId },
    WaitObservations,
    QuitOptimization,
}
