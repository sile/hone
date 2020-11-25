use crate::envvar;
use crate::event::{Event, EventReader, EventWriter, ObservationEvent, StudyEvent, TrialEvent};
use crate::metric::MetricInstance;
use crate::param::{ParamInstance, ParamValue};
use crate::rpc;
use crate::study::{CommandSpec, StudySpec};
use crate::trial::{Observation, ObservationId, TrialId};
use crate::tuners::{Action, Tune, Tuner};
use crate::types::Scope;
use anyhow::Context;
use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

mod loader;

pub fn spawn(
    spec: &CommandSpec,
    observation_id: ObservationId,
    rpc_server_addr: std::net::SocketAddr,
) -> anyhow::Result<CommandRunner> {
    let mut command = Command::new(&spec.path);
    // TODO: trial_id and study_instance_id envs
    command
        .args(&spec.args)
        .env(envvar::KEY_SERVER_ADDR, rpc_server_addr.to_string())
        .env(envvar::KEY_OBSERVATION_ID, observation_id.get().to_string())
        .stdin(Stdio::null());
    let proc = command
        .spawn()
        .with_context(|| format!("Failed to spawn command: {:?}", spec.path))?;
    Ok(CommandRunner {
        observation_id,
        proc,
    })
}

#[derive(Debug)]
pub struct CommandRunner {
    observation_id: ObservationId,
    proc: Child,
}

impl CommandRunner {
    pub fn try_wait(&mut self) -> anyhow::Result<Option<ExitStatus>> {
        let exit_status = self.proc.try_wait()?;
        Ok(exit_status)
    }
}

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
    study_temp_dir: Option<tempfile::TempDir>,
    trial_temp_dirs: HashMap<TrialId, tempfile::TempDir>,
    obs_temp_dirs: HashMap<ObservationId, tempfile::TempDir>,
    elapsed_offset: Duration,
}

impl<W: Write> StudyRunner<W> {
    pub fn new(output: W, opt: StudyRunnerOpt) -> anyhow::Result<Self> {
        let tuner = opt.study.tuner.build()?;
        let (rpc_server_addr, rpc_channel) = rpc::spawn_rpc_server()?;

        let mut output = EventWriter::new(output);
        output.write(Event::Study(StudyEvent::Started))?;
        output.write(Event::Study(StudyEvent::Defined {
            spec: opt.study.clone(),
        }))?;

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
            study_temp_dir: None,
            trial_temp_dirs: HashMap::new(),
            obs_temp_dirs: HashMap::new(),
            elapsed_offset: Duration::new(0, 0),
        })
    }

    fn start_observation(&mut self, obs: Observation) -> anyhow::Result<()> {
        self.output
            .write(Event::Observation(ObservationEvent::Started {
                obs_id: obs.id,
                trial_id: obs.trial_id,
                elapsed: self.start_time.elapsed(),
            }))?;
        self.runnings.push(spawn(
            &self.opt.study.command,
            obs.id,
            self.rpc_server_addr,
        )?);
        self.observations.insert(obs.id, obs);
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
                        self.output.write(Event::Trial(TrialEvent::Started {
                            trial_id: obs.trial_id,
                        }))?;
                        self.start_observation(obs)?;
                        did_nothing = false;
                        break;
                    }
                    Action::ResumeTrial { trial_id } => {
                        let obs =
                            Observation::new(self.next_obs_id.fetch_and_increment(), trial_id);
                        self.start_observation(obs)?;
                        did_nothing = false;
                        break;
                    }
                    Action::FinishTrial { trial_id } => {
                        self.output
                            .write(Event::Trial(TrialEvent::Finished { trial_id }))?;
                        self.trial_temp_dirs.remove(&trial_id);
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
                    let mut obs = self
                        .observations
                        .remove(&finished.observation_id)
                        .expect("bug");
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
        let elapsed = self.elapsed_offset + elapsed;
        let trial_id = obs.trial_id;
        let trial_finished = self.tuner.tell(&obs)?;
        self.obs_temp_dirs.remove(&obs.id);
        self.output
            .write(Event::observation_finished(obs, elapsed))?;
        if trial_finished {
            self.trial_temp_dirs.remove(&trial_id);
            self.output.write(Event::trial_finished(trial_id))?;
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

    fn ensure_temp_dir_created(
        tempdir: Option<&tempfile::TempDir>,
        parent: Option<&PathBuf>,
    ) -> anyhow::Result<(Option<tempfile::TempDir>, PathBuf)> {
        if let Some(temp) = tempdir {
            Ok((None, temp.path().to_path_buf()))
        } else if let Some(parent) = parent {
            std::fs::create_dir_all(parent)?;
            let temp = tempfile::TempDir::new_in(parent)?;
            let path = temp.path().to_path_buf();
            Ok((Some(temp), path))
        } else {
            let temp = tempfile::TempDir::new()?;
            let path = temp.path().to_path_buf();
            Ok((Some(temp), path))
        }
    }

    fn handle_mktemp(&mut self, req: rpc::MktempReq) -> anyhow::Result<PathBuf> {
        match req.scope {
            Scope::Study => {
                let (temp, path) = Self::ensure_temp_dir_created(
                    self.study_temp_dir.as_ref(),
                    req.parent.as_ref(),
                )?;
                if let Some(temp) = temp {
                    self.study_temp_dir = Some(temp)
                }
                Ok(path)
            }
            Scope::Observation => {
                let (temp, path) = Self::ensure_temp_dir_created(
                    self.obs_temp_dirs.get(&req.observation_id),
                    req.parent.as_ref(),
                )?;
                if let Some(temp) = temp {
                    self.obs_temp_dirs.insert(req.observation_id, temp);
                }
                Ok(path)
            }
            _ => {
                let trial_id = self
                    .observations
                    .get(&req.observation_id)
                    .ok_or_else(|| {
                        anyhow::anyhow!("unknown observation: {:?}", req.observation_id)
                    })?
                    .trial_id;
                let (temp, path) = Self::ensure_temp_dir_created(
                    self.trial_temp_dirs.get(&trial_id),
                    req.parent.as_ref(),
                )?;
                if let Some(temp) = temp {
                    self.trial_temp_dirs.insert(trial_id, temp);
                }
                Ok(path)
            }
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
