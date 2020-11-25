use self::command::CommandRunner;
use self::tempdir::TempDirs;
use crate::event::{Event, EventReader, EventWriter};
use crate::metric::MetricInstance;
use crate::param::{ParamInstance, ParamValue};
use crate::rpc;
use crate::study::StudySpec;
use crate::trial::{Observation, ObservationId, TrialId};
use crate::tuners::{Action, Tune, Tuner};
use crate::types::Scope;
use std::io::{BufRead, Write};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::time::{Duration, Instant};

mod command;
mod loader;
mod tempdir;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StudyRunnerOpt {
    pub study: StudySpec,
    pub workers: NonZeroUsize,
    pub repeat: Option<usize>,
}

#[derive(Debug)]
pub struct StudyRunner<W> {
    output: EventWriter<W>,
    runnings: Vec<CommandRunner>,
    next_obs_id: ObservationId,
    next_trial_id: TrialId,
    rpc_channel: rpc::Channel,
    tuner: Tuner,
    opt: StudyRunnerOpt,
    start_time: Instant,
    elapsed_offset: Duration,
    tempdirs: TempDirs,
    terminating: bool,
}

impl<W: Write> StudyRunner<W> {
    pub fn new(output: W, opt: StudyRunnerOpt) -> anyhow::Result<Self> {
        let tuner = opt.study.tuner.build()?;
        let rpc_channel = rpc::spawn_rpc_server()?;

        let mut output = EventWriter::new(output);
        output.write(Event::study_started())?;
        output.write(Event::study_defined(opt.study.clone()))?;

        Ok(Self {
            output,
            runnings: Vec::new(),
            rpc_channel,
            next_obs_id: ObservationId::new(0),
            next_trial_id: TrialId::new(0),
            tuner,
            opt,
            start_time: Instant::now(),
            tempdirs: TempDirs::new(),
            elapsed_offset: Duration::new(0, 0),
            terminating: false,
        })
    }

    pub fn load_study<R: BufRead>(&mut self, reader: EventReader<R>) -> anyhow::Result<()> {
        let mut loader = self::loader::StudyLoader::new(self);
        loader.load(reader)
    }

    // TODO: add signal handling
    pub fn run(mut self) -> anyhow::Result<()> {
        self.start_time = Instant::now();

        let mut finished_count = 0;
        let mut did_nothing;
        while self.opt.repeat.map_or(true, |n| finished_count < n) {
            did_nothing = true;

            while self.runnings.len() < self.opt.workers.get() && !self.terminating {
                let action = self.tuner.next_action()?;
                let waiting = matches!(action, Action::WaitObservations);
                self.handle_action(action)?;
                if waiting {
                    break;
                } else {
                    did_nothing = false;
                }
            }

            while let Some(message) = self.rpc_channel.try_recv() {
                self.handle_message(message)?;
                did_nothing = false;
            }

            let mut i = 0;
            while i < self.runnings.len() {
                if self.runnings[i].is_exited()? {
                    finished_count += 1;
                    let obs = self.runnings.swap_remove(i).into_obs();
                    self.tell_finished_obs(obs, self.start_time.elapsed())?;
                    did_nothing = false;
                } else {
                    i += 1;
                }
            }

            if did_nothing {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        Ok(())
    }

    fn tell_finished_obs(&mut self, obs: Observation, elapsed: Duration) -> anyhow::Result<()> {
        let trial_id = obs.trial_id;
        let trial_finished = self.tuner.tell(&obs)?;
        self.finish_obs(obs, elapsed)?;
        if trial_finished {
            self.finish_trial(trial_id)?;
        }
        Ok(())
    }

    fn start_obs(&mut self, obs: Observation) -> anyhow::Result<()> {
        self.output.write(Event::observation_started(
            obs.id,
            obs.trial_id,
            self.elapsed_offset + self.start_time.elapsed(),
        ))?;
        self.runnings.push(CommandRunner::spawn(
            &self.opt.study,
            obs,
            self.rpc_channel.server_addr,
        )?);
        Ok(())
    }

    fn finish_obs(&mut self, obs: Observation, elapsed: Duration) -> anyhow::Result<()> {
        let elapsed = self.elapsed_offset + elapsed;
        self.tempdirs.remove_obs_tempdir(obs.id);
        self.output
            .write(Event::observation_finished(obs, elapsed))?;
        Ok(())
    }

    fn start_trial(&mut self, trial_id: TrialId) -> anyhow::Result<()> {
        self.output.write(Event::trial_started(trial_id))?;
        Ok(())
    }

    fn finish_trial(&mut self, trial_id: TrialId) -> anyhow::Result<()> {
        self.tempdirs.remove_trial_tempdir(trial_id);
        self.output.write(Event::trial_finished(trial_id))?;
        Ok(())
    }

    fn handle_action(&mut self, action: Action) -> anyhow::Result<()> {
        match action {
            Action::CreateTrial => {
                let obs = Observation::new(
                    self.next_obs_id.fetch_and_increment(),
                    self.next_trial_id.fetch_and_increment(),
                );
                self.start_trial(obs.trial_id)?;
                self.start_obs(obs)?;
            }
            Action::ResumeTrial { trial_id } => {
                let obs = Observation::new(self.next_obs_id.fetch_and_increment(), trial_id);
                self.start_obs(obs)?;
            }
            Action::FinishTrial { trial_id } => {
                self.finish_trial(trial_id)?;
            }
            Action::WaitObservations => {}
            Action::QuitOptimization => {
                self.terminating = true;
                for worker in &mut self.runnings {
                    worker.kill()?;
                }
            }
        }
        Ok(())
    }

    fn handle_message(&mut self, message: rpc::Message) -> anyhow::Result<()> {
        match message {
            rpc::Message::Ask { req, reply } => {
                let value = self.handle_ask(req)?;
                reply.send(value)?;
            }
            rpc::Message::Tell { req, reply } => {
                self.handle_tell(req)?;
                reply.send(())?;
            }
            rpc::Message::Mktemp { req, reply } => {
                let path = self.handle_mktemp(req)?;
                reply.send(path)?;
            }
        }
        Ok(())
    }

    fn handle_mktemp(&mut self, req: rpc::MktempReq) -> anyhow::Result<PathBuf> {
        match req.scope {
            Scope::Study => self.tempdirs.create_study_tempdir(req.parent.as_ref()),
            Scope::Trial => {
                let trial_id = self
                    .runnings
                    .iter()
                    .find(|o| o.obs().id == req.observation_id)
                    .ok_or_else(|| {
                        anyhow::anyhow!("unknown observation: {:?}", req.observation_id)
                    })?
                    .obs()
                    .trial_id;
                self.tempdirs
                    .create_trial_tempdir(trial_id, req.parent.as_ref())
            }
            Scope::Observation => self
                .tempdirs
                .create_obs_tempdir(req.observation_id, req.parent.as_ref()),
        }
    }

    fn handle_ask(&mut self, req: rpc::AskReq) -> anyhow::Result<ParamValue> {
        let obs = self
            .runnings
            .iter_mut()
            .find(|o| o.obs().id == req.observation_id)
            .ok_or_else(|| anyhow::anyhow!("unknown observation_id {}", req.observation_id.get()))?
            .obs_mut();
        if let Some(instance) = obs.params.get(&req.param_name) {
            Ok(instance.value.clone())
        } else {
            let value = self.tuner.ask(obs, &req.param_name, &req.param_type)?;
            obs.params.insert(
                req.param_name,
                ParamInstance::new(req.param_type, value.clone()),
            );
            Ok(value)
        }
    }

    fn handle_tell(&mut self, req: rpc::TellReq) -> anyhow::Result<()> {
        let obs = self
            .runnings
            .iter_mut()
            .find(|o| o.obs().id == req.observation_id)
            .ok_or_else(|| anyhow::anyhow!("unknown observation_id {}", req.observation_id.get()))?
            .obs_mut();
        obs.metrics.insert(
            req.metric_name,
            MetricInstance::new(req.metric_type, req.metric_value),
        );
        Ok(())
    }
}
