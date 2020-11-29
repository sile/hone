use crate::param::{ParamName, ParamType, ParamValue};
use crate::trial::{Observation, TrialId};
use crate::tuners::{Action, ActionQueue, Tune, Tuner};
use std::collections::HashMap;

#[derive(Debug)]
pub struct RetryTuner {
    tuner: Tuner,
    max_retries: usize,
    actions: ActionQueue,
    retryings: HashMap<TrialId, FailedObservation>,
}

impl RetryTuner {
    pub fn new(tuner: Tuner, max_retries: usize) -> Self {
        Self {
            tuner,
            max_retries,
            actions: ActionQueue::new(),
            retryings: HashMap::new(),
        }
    }
}

impl Tune for RetryTuner {
    fn ask(
        &mut self,
        obs: &Observation,
        param_name: &ParamName,
        param_type: &ParamType,
    ) -> anyhow::Result<ParamValue> {
        if let Some(failed) = self.retryings.get(&obs.trial_id) {
            let param_value = failed
                .obs
                .params
                .get(param_name)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "retried trial asked a different parameter: {:?}",
                        param_name
                    )
                })?
                .value
                .clone();
            Ok(param_value)
        } else {
            self.tuner.ask(obs, param_name, param_type)
        }
    }

    fn tell(&mut self, obs: &Observation) -> anyhow::Result<()> {
        assert!(obs.exit_status.is_some());

        if obs.is_succeeded() {
            let obs = if let Some(mut orig_obs) =
                self.retryings.remove(&obs.trial_id).map(|x| x.obs)
            {
                anyhow::ensure!(
                    orig_obs.params == obs.params,
                    "retried trial has the different parameters with the original one: retried={:?}, original={:?}",
                    obs.params, orig_obs.params);
                orig_obs.metrics = obs.metrics.clone();
                orig_obs.exit_status = obs.exit_status;
                orig_obs
            } else {
                obs.clone()
            };
            return self.tuner.tell(&obs);
        }

        let failed = self
            .retryings
            .entry(obs.trial_id)
            .or_insert_with(|| FailedObservation {
                obs: obs.clone(),
                retried_count: 0,
            });
        if failed.retried_count < self.max_retries {
            failed.retried_count += 1;
            self.actions.enqueue(Action::resume_trial(obs.trial_id));
        } else {
            self.tuner.tell(&failed.obs)?;
        }
        Ok(())
    }

    fn next_action(&mut self) -> Option<Action> {
        self.actions.next().or_else(|| self.tuner.next_action())
    }
}

#[derive(Debug)]
struct FailedObservation {
    obs: Observation,
    retried_count: usize,
}
