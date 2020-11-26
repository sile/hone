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
enum TunerSpecInner {
    // TODO:  HyperbandTuner, TpeTuner
    Random(self::random::RandomTunerSpec),
}

impl TunerSpecInner {
    pub fn build(&self) -> anyhow::Result<Tuner> {
        match self {
            Self::Random(spec) => spec.build().map(Tuner::new),
        }
    }
}

impl Default for TunerSpecInner {
    fn default() -> Self {
        Self::Random(self::random::RandomTunerSpec::default())
    }
}

#[derive(Debug, Default, Clone, structopt::StructOpt, serde::Serialize, serde::Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "snake_case")]
pub struct TunerSpec {
    #[structopt(long, default_value = "0")]
    retry: usize,

    // TODO: RetryTuner, AverageTuner, HyperbandTuner, TpeTuner
    #[structopt(flatten)]
    #[serde(flatten)]
    inner: TunerSpecInner,
}

impl TunerSpec {
    pub fn build(&self) -> anyhow::Result<Tuner> {
        if self.retry == 0 {
            self.inner.build()
        } else {
            todo!()
        }
    }
}

pub struct Tuner(Box<dyn 'static + Tune + Send + Sync>);

impl Tuner {
    pub fn new<T>(tuner: T) -> Self
    where
        T: 'static + Tune + Send + Sync,
    {
        Self(Box::new(tuner))
    }
}

impl Tune for Tuner {
    fn ask(
        &mut self,
        obs: &Observation,
        param_name: &ParamName,
        param_type: &ParamType,
    ) -> anyhow::Result<ParamValue> {
        self.0.ask(obs, param_name, param_type)
    }

    fn tell(&mut self, obs: &Observation) -> anyhow::Result<()> {
        self.0.tell(obs)
    }

    fn next_action(&mut self) -> anyhow::Result<Action> {
        self.0.next_action()
    }
}

impl std::fmt::Debug for Tuner {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Tuner {{ .. }}")
    }
}
