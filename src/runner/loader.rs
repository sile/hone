use super::StudyRunner;
use crate::event::{Event, EventReader, ObservationEvent, StudyEvent, TrialEvent};
use crate::trial::{ObservationId, TrialId};
use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::time::Duration;

#[derive(Debug)]
pub struct StudyLoader<'a, W> {
    study: &'a mut StudyRunner<W>,
    trial_id_mapping: HashMap<TrialId, TrialId>,
    obs_id_mapping: HashMap<ObservationId, ObservationId>,
    last_elapsed: Duration,
}

impl<'a, W: Write> StudyLoader<'a, W> {
    pub fn new(study: &'a mut StudyRunner<W>) -> Self {
        Self {
            study,
            trial_id_mapping: HashMap::new(),
            obs_id_mapping: HashMap::new(),
            last_elapsed: Duration::new(0, 0),
        }
    }

    pub fn load<R: BufRead>(&mut self, mut reader: EventReader<R>) -> anyhow::Result<()> {
        let mut skip = true;
        while let Some(event) = reader.read()? {
            if let Some(elapsed) = event.elapsed() {
                self.last_elapsed = elapsed;
            }

            match event {
                Event::Study(StudyEvent::Started) => {
                    skip = true;
                    self.study.elapsed_offset += self.last_elapsed;
                    self.trial_id_mapping = HashMap::new();
                    self.obs_id_mapping = HashMap::new();
                    self.last_elapsed = Duration::new(0, 0);
                }
                Event::Study(StudyEvent::Defined { .. }) => {
                    skip = false;
                }
                Event::Trial(event) if !skip => {
                    self.handle_trial_event(event)?;
                }
                Event::Observation(event) if !skip => {
                    self.handle_observation_event(event)?;
                }
                _ => {}
            }
        }
        self.study.elapsed_offset += self.last_elapsed;

        Ok(())
    }

    fn handle_trial_event(&mut self, event: TrialEvent) -> anyhow::Result<()> {
        if let TrialEvent::Started {
            trial_id: orig_trial_id,
        } = event
        {
            let trial_id = self.study.next_trial_id.fetch_and_increment();
            self.trial_id_mapping.insert(orig_trial_id, trial_id);
            self.study.output.write(Event::trial_started(trial_id))?;
        }
        Ok(())
    }

    fn handle_observation_event(&mut self, event: ObservationEvent) -> anyhow::Result<()> {
        match event {
            ObservationEvent::Started {
                obs_id: orig_obs_id,
                trial_id: orig_trial_id,
                elapsed,
            } => {
                let obs_id = self.study.next_obs_id.fetch_and_increment();
                let trial_id = *self
                    .trial_id_mapping
                    .get(&orig_trial_id)
                    .ok_or_else(|| anyhow::anyhow!("unknown trial id {:?}", orig_trial_id))?;
                self.obs_id_mapping.insert(orig_obs_id, obs_id);
                let elapsed = self.study.elapsed_offset + elapsed;
                self.study
                    .output
                    .write(Event::observation_started(obs_id, trial_id, elapsed))?;
            }
            ObservationEvent::Finished { mut obs, elapsed } => {
                let obs_id = *self
                    .obs_id_mapping
                    .get(&obs.id)
                    .ok_or_else(|| anyhow::anyhow!("unknown observation id {:?}", obs.id))?;
                let trial_id = *self
                    .trial_id_mapping
                    .get(&obs.trial_id)
                    .ok_or_else(|| anyhow::anyhow!("unknown trial id {:?}", obs.trial_id))?;
                obs.id = obs_id;
                obs.trial_id = trial_id;
                self.study.tell_finished_obs(obs, elapsed)?;
            }
        }
        Ok(())
    }
}
