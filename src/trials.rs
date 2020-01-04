use crate::config::Config;
use crate::pubsub::{PubSub, TrialAction};
use crate::trial::Trial;
use crate::{ErrorKind, Result};
use std::collections::HashMap;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct TrialsOpt {
    pub study_name: String,
}

pub fn list_trials(opt: TrialsOpt, config: &Config) -> Result<Vec<Trial>> {
    let data_dir = track!(config.data_dir())?;
    let mut pubsub = PubSub::new(data_dir);
    let mut subscriber = track!(pubsub.subscribe(&opt.study_name))?;

    let mut trials = HashMap::new();
    for (id, action) in track!(subscriber.poll())? {
        match action {
            TrialAction::Start { id, timestamp } => {
                trials.insert(id, Trial::with_id_and_timestamp(id, timestamp));
            }
            TrialAction::Define { param } => {
                let trial = track_assert_some!(trials.get_mut(&id), ErrorKind::InvalidInput);
                trial.param_specs.insert(param.name, param.spec);
            }
            TrialAction::Sample { name, value } => {
                let trial = track_assert_some!(trials.get_mut(&id), ErrorKind::InvalidInput);
                trial.param_values.insert(name, value);
            }
            TrialAction::Report { step, metric } => {
                let trial = track_assert_some!(trials.get_mut(&id), ErrorKind::InvalidInput);
                trial.report(Some(step), &metric);
            }
            TrialAction::End => {}
        }
    }

    Ok(trials.into_iter().map(|(_, v)| v).collect())
}
