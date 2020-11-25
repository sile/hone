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
use std::collections::HashMap;
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
    observations: HashMap<ObservationId, Observation>,
    finished_observations: usize,
    next_obs_id: ObservationId,
    next_trial_id: TrialId,
    rpc_server_addr: std::net::SocketAddr,
    rpc_channel: rpc::Channel,
    tuner: Tuner,
    opt: StudyRunnerOpt,
    start_time: Instant,
    tempdirs: TempDirs,
    elapsed_offset: Duration,
}

impl<W: Write> StudyRunner<W> {
    pub fn new(output: W, opt: StudyRunnerOpt) -> anyhow::Result<Self> {
        let tuner = opt.study.tuner.build()?;
        let (rpc_server_addr, rpc_channel) = rpc::spawn_rpc_server()?;

        let mut output = EventWriter::new(output);
        output.write(Event::study_started())?;
        output.write(Event::study_defined(opt.study.clone()))?;

        Ok(Self {
            output,
            runnings: Vec::new(),
            observations: HashMap::new(),
            finished_observations: 0,
            rpc_server_addr,
            rpc_channel,
            next_obs_id: ObservationId::new(0),
            next_trial_id: TrialId::new(0),
            tuner,
            opt,
            start_time: Instant::now(),
            tempdirs: TempDirs::new(),
            elapsed_offset: Duration::new(0, 0),
        })
    }

    fn start_obs(&mut self, obs: Observation) -> anyhow::Result<()> {
        self.output.write(Event::observation_started(
            obs.id,
            obs.trial_id,
            self.elapsed_offset + self.start_time.elapsed(),
        ))?;
        self.runnings.push(CommandRunner::spawn(
            &self.opt.study,
            &obs,
            self.rpc_server_addr,
        )?);
        self.observations.insert(obs.id, obs);
        Ok(())
    }

    fn finish_obs(&mut self, obs: Observation, elapsed: Duration) -> anyhow::Result<()> {
        let elapsed = self.elapsed_offset + elapsed;
        self.tempdirs.remove_obs_tempdir(obs.id);
        self.output
            .write(Event::observation_finished(obs, elapsed))?;
        Ok(())
    }

    fn finish_trial(&mut self, trial_id: TrialId) -> anyhow::Result<()> {
        self.tempdirs.remove_trial_tempdir(trial_id);
        self.output.write(Event::trial_finished(trial_id))?;
        Ok(())
    }

    pub fn load_study<R: BufRead>(&mut self, reader: EventReader<R>) -> anyhow::Result<()> {
        let mut loader = self::loader::StudyLoader::new(self);
        loader.load(reader)
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        self.start_time = Instant::now();

        let mut did_nothing;
        while !self.is_study_finished() {
            did_nothing = true;

            while self.runnings.len() < self.opt.workers.get() {
                match self.tuner.next_action()? {
                    Action::CreateTrial => {
                        let obs = Observation::new(
                            self.next_obs_id.fetch_and_increment(),
                            self.next_trial_id.fetch_and_increment(),
                        );
                        self.output.write(Event::trial_started(obs.trial_id))?;
                        self.start_obs(obs)?;
                        did_nothing = false;
                        break;
                    }
                    Action::ResumeTrial { trial_id } => {
                        let obs =
                            Observation::new(self.next_obs_id.fetch_and_increment(), trial_id);
                        self.start_obs(obs)?;
                        did_nothing = false;
                        break;
                    }
                    Action::FinishTrial { trial_id } => {
                        self.finish_trial(trial_id)?;
                    }
                    Action::WaitObservations => {
                        break;
                    }
                    Action::QuitOptimization => {
                        // TODO: kill running processes
                        return Ok(());
                    }
                }
            }

            while let Some(message) = self.rpc_channel.try_recv() {
                self.handle_message(message)?;
                did_nothing = false;
            }

            let mut i = 0;
            while i < self.runnings.len() {
                if let Some(status) = self.runnings[i].try_wait()? {
                    let finished = self.runnings.swap_remove(i);
                    self.finished_observations += 1;
                    let mut obs = self.observations.remove(&finished.obs_id()).expect("bug");
                    obs.exit_status = status.code();
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

    fn is_study_finished(&self) -> bool {
        self.opt
            .repeat
            .map_or(false, |n| self.finished_observations >= n)
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
                    .observations
                    .get(&req.observation_id)
                    .ok_or_else(|| {
                        anyhow::anyhow!("unknown observation: {:?}", req.observation_id)
                    })?
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
            .observations
            .get_mut(&req.observation_id)
            .ok_or_else(|| {
                anyhow::anyhow!("unknown observation_id {}", req.observation_id.get())
            })?;
        // TODO: check whether the parameter has already been asked.
        let value = self.tuner.ask(obs, &req.param_name, &req.param_type)?;
        obs.params.insert(
            req.param_name,
            ParamInstance::new(req.param_type, value.clone()),
        );
        Ok(value)
    }

    fn handle_tell(&mut self, req: rpc::TellReq) -> anyhow::Result<()> {
        let obs = self
            .observations
            .get_mut(&req.observation_id)
            .ok_or_else(|| {
                anyhow::anyhow!("unknown observation_id {}", req.observation_id.get())
            })?;
        obs.metrics.insert(
            req.metric_name,
            MetricInstance::new(req.metric_type, req.metric_value),
        );
        Ok(())
    }
}
