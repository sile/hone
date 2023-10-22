use crate::param::{ParamName, ParamType, ParamValue};
use crate::trial::{Observation, TrialId};
use std::collections::VecDeque;

pub mod random;
pub mod retry;

pub trait Tune {
    fn ask(
        &mut self,
        obs: &Observation,
        param_name: &ParamName,
        param_type: &ParamType,
    ) -> anyhow::Result<ParamValue>;

    fn tell(&mut self, obs: &Observation) -> anyhow::Result<()>;

    fn next_action(&mut self) -> Option<Action>;
}

#[derive(Debug, Clone)]
pub enum Action {
    ResumeTrial { trial_id: TrialId },
    FinishTrial { trial_id: TrialId },
    WaitObservations,
    QuitOptimization,
}

impl Action {
    pub const fn resume_trial(trial_id: TrialId) -> Self {
        Self::ResumeTrial { trial_id }
    }

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

    pub fn next(&mut self) -> Option<Action> {
        self.0.pop_front()
    }
}

#[derive(Debug, Clone, clap::Subcommand, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Default, Clone, clap::Args, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TunerSpec {
    #[clap(long, default_value = "0")]
    retry: usize,

    // TODO: AverageTuner, HyperbandTuner, TpeTuner
    #[clap(subcommand)]
    #[serde(flatten)]
    inner: Option<TunerSpecInner>,
}

impl TunerSpec {
    pub fn build(&self) -> anyhow::Result<Tuner> {
        let default_tuner = TunerSpecInner::Random(self::random::RandomTunerSpec::default());
        let mut tuner = self.inner.as_ref().unwrap_or(&default_tuner).build()?;
        if self.retry > 0 {
            tuner = Tuner::new(self::retry::RetryTuner::new(tuner, self.retry));
        }
        Ok(tuner)
    }
}

impl std::str::FromStr for TunerSpec {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::json::parse_json(s)
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

    fn next_action(&mut self) -> Option<Action> {
        self.0.next_action()
    }
}

impl std::fmt::Debug for Tuner {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Tuner {{ .. }}")
    }
}
