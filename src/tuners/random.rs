use crate::param::{NumParamType, ParamName, ParamType, ParamValue, StrParamType};
use crate::rng::ArcRng;
use crate::trial::{Observation, TrialId};
use crate::tuners::Optimize;
use crate::types::FiniteF64;
use rand::distributions::Distribution;
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Debug, Clone, Default, structopt::StructOpt, serde::Serialize, serde::Deserialize)]
pub struct RandomTunerSpec {
    #[structopt(long)]
    pub seed: Option<u64>,
}

impl RandomTunerSpec {
    pub fn build(&self) -> anyhow::Result<RandomTuner> {
        let rng = ArcRng::new(self.seed);
        Ok(RandomTuner::new(rng))
    }
}

#[derive(Debug)]
pub struct RandomTuner {
    rng: ArcRng,
    finished_trials: Vec<TrialId>,
}

impl RandomTuner {
    pub fn new(rng: ArcRng) -> Self {
        RandomTuner {
            rng,
            finished_trials: Vec::new(),
        }
    }
}

impl Optimize for RandomTuner {
    fn ask(
        &mut self,
        _obs: &Observation,
        _param_name: &ParamName,
        param_type: &ParamType,
    ) -> anyhow::Result<ParamValue> {
        let rng = &mut self.rng;
        match param_type {
            ParamType::Str(StrParamType::Categorical(t)) => Ok(ParamValue::Str(
                t.choices().get().choose(rng).expect("unreachable").clone(),
            )),
            ParamType::Str(StrParamType::Ordinal(t)) => Ok(ParamValue::Str(
                t.choices().get().choose(rng).expect("unreachable").clone(),
            )),
            ParamType::Num(NumParamType::Continous(t)) => {
                if t.ln() {
                    let v = rng.gen_range(t.range().min().get().ln(), t.range().max().get().ln());
                    let v = FiniteF64::new(v.exp())?;
                    Ok(ParamValue::Num(v))
                } else {
                    let v = rng.gen_range(t.range().min().get(), t.range().max().get());
                    Ok(ParamValue::Num(FiniteF64::new(v).expect("unreachable")))
                }
            }
            ParamType::Num(NumParamType::Discrete(t)) => {
                let n = rng.gen_range(0, t.count());
                let v = t.range().min().get() + t.step().get() * n as f64;
                let v = FiniteF64::new(v)?;
                Ok(ParamValue::Num(v))
            }
            ParamType::Num(NumParamType::Normal(t)) => {
                let d = rand_distr::Normal::new(t.mean().get(), t.stddev().get())?;
                let v = d.sample(rng);
                let v = FiniteF64::new(v)?;
                Ok(ParamValue::Num(v))
            }
            ParamType::Num(NumParamType::Fidelity(t)) => Ok(ParamValue::Num(t.range().max())),
        }
    }

    fn tell(&mut self, obs: &Observation) -> anyhow::Result<bool> {
        self.finished_trials.push(obs.trial_id);
        Ok(true)
    }
}
