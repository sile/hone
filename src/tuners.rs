use crate::param::{ParamName, ParamType, ParamValue};
use crate::trial::{Observation, TrialId};
use std::collections::VecDeque;

pub mod random;

pub trait Tune {
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

impl Action {
    pub const fn finish_trial(trial_id: TrialId) -> Self {
        Self::FinishTrial { trial_id }
    }
}

#[derive(Debug)]
pub struct ActionQueue(VecDeque<Action>);

impl ActionQueue {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    pub fn enqueue(&mut self, action: Action) {
        self.0.push_back(action);
    }

    pub fn next(&mut self) -> Action {
        self.0.pop_front().unwrap_or_else(|| Action::CreateTrial)
    }
}

#[derive(Debug, Clone, structopt::StructOpt, serde::Serialize, serde::Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "snake_case")]
pub enum TunerSpec {
    // TODO: RetryTuner, AverageTuner, HyperbandTuner, TpeTuner
    Random(self::random::RandomTunerSpec),
}

impl TunerSpec {
    pub fn build(&self) -> anyhow::Result<Tuner> {
        match self {
            Self::Random(spec) => spec.build().map(Tuner::Random),
        }
    }
}

impl Default for TunerSpec {
    fn default() -> Self {
        Self::Random(self::random::RandomTunerSpec::default())
    }
}

#[derive(Debug)]
pub enum Tuner {
    Random(self::random::RandomTuner),
}

impl Tune for Tuner {
    fn ask(
        &mut self,
        obs: &Observation,
        param_name: &ParamName,
        param_type: &ParamType,
    ) -> anyhow::Result<ParamValue> {
        match self {
            Self::Random(o) => o.ask(obs, param_name, param_type),
        }
    }

    fn tell(&mut self, obs: &Observation) -> anyhow::Result<()> {
        match self {
            Self::Random(o) => o.tell(obs),
        }
    }

    fn next_action(&mut self) -> anyhow::Result<Action> {
        match self {
            Self::Random(o) => o.next_action(),
        }
    }
}
