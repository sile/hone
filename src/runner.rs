use crate::envvar;
use crate::event::{Event, EventReader, EventWriter, ObservationEvent, StudyEvent, TrialEvent};
use crate::metric::MetricInstance;
use crate::optimizer::Action;
use crate::optimizer::{Optimize, Optimizer};
use crate::param::{ParamInstance, ParamValue};
use crate::rpc;
use crate::trial::{Observation, ObservationId, TrialId};
use crate::types::Scope;
use anyhow::Context;
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommandRunnerOpt {
    pub path: String,
    pub args: Vec<String>,
}

impl CommandRunnerOpt {
    pub fn spawn(
        &self,
        observation_id: ObservationId,
        rpc_server_addr: std::net::SocketAddr,
    ) -> anyhow::Result<CommandRunner> {
        let mut command = Command::new(&self.path);
        // TODO: trial_id and study_instance_id envs
        command
            .args(&self.args)
            .env(envvar::KEY_SERVER_ADDR, rpc_server_addr.to_string())
            .env(envvar::KEY_OBSERVATION_ID, observation_id.get().to_string())
            .stdin(Stdio::null());
        let proc = command
            .spawn()
            .with_context(|| format!("Failed to spawn command: {:?}", self.path))?;
        Ok(CommandRunner {
            observation_id,
            proc,
        })
    }
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
    pub study_name: String,
    pub study_instance: uuid::Uuid,
    pub resume: Option<PathBuf>,
    pub attrs: BTreeMap<String, String>,
    pub workers: NonZeroUsize,
    pub runs: Option<usize>,
    pub command: CommandRunnerOpt,
    pub output: Option<PathBuf>,
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
    optimizer: Optimizer,
    opt: StudyRunnerOpt,
    start_time: Instant,
    study_temp_dir: Option<tempfile::TempDir>,
    trial_temp_dirs: HashMap<TrialId, tempfile::TempDir>,
    obs_temp_dirs: HashMap<ObservationId, tempfile::TempDir>,
}

impl<W: Write> StudyRunner<W> {
    pub fn new(output: W, optimizer: Optimizer, opt: StudyRunnerOpt) -> anyhow::Result<Self> {
        let (rpc_server_addr, rpc_channel) = rpc::spawn_rpc_server()?;
        Ok(Self {
            output: EventWriter::new(output),
            runnings: Vec::new(),
            observations: HashMap::new(),
            finished_observations: 0,
            rpc_server_addr,
            rpc_channel,
            next_obs_id: ObservationId::new(0),
            next_trial_id: TrialId::new(0),
            optimizer,
            opt,
            start_time: Instant::now(),
            study_temp_dir: None,
            trial_temp_dirs: HashMap::new(),
            obs_temp_dirs: HashMap::new(),
        })
    }

    fn start_observation(&mut self, obs: Observation) -> anyhow::Result<()> {
        self.output.write(Event::Obs(ObservationEvent::Started {
            obs_id: obs.id,
            trial_id: obs.trial_id,
            elapsed: self.start_time.elapsed(),
        }))?;
        self.runnings
            .push(self.opt.command.spawn(obs.id, self.rpc_server_addr)?);
        self.observations.insert(obs.id, obs);
        Ok(())
    }

    fn resume_study(&mut self, path: PathBuf) -> anyhow::Result<()> {
        let file =
            std::fs::File::open(&path).with_context(|| format!("Cannot open file: {:?}", path))?;
        let mut reader = EventReader::new(std::io::BufReader::new(file));
        while let Some(event) = reader.read()? {
            match &event {
                Event::Study(_) => {
                    continue;
                }
                Event::Trial(e) => match e {
                    TrialEvent::Started { trial_id, .. } => {
                        self.next_trial_id = TrialId::new(trial_id.get() + 1);
                    }
                    _ => {}
                },
                Event::Obs(e) => match e {
                    ObservationEvent::Started { obs_id, .. } => {
                        self.next_obs_id = ObservationId::new(obs_id.get() + 1);
                    }
                    ObservationEvent::Finished { obs, .. } => {
                        self.optimizer.tell(&obs)?;
                    }
                },
            }
            self.output.write(event)?;
        }
        Ok(())
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        self.output.write(Event::Study(StudyEvent::Defined {
            opt: self.opt.clone(),
        }))?;
        self.output.write(Event::Study(StudyEvent::Started))?;
        if let Some(path) = self.opt.resume.clone() {
            self.resume_study(path)?;
        }
        self.start_time = Instant::now();

        let mut did_nothing;
        while !self.is_study_finished() {
            did_nothing = true;

            while self.runnings.len() < self.opt.workers.get() {
                match self.optimizer.next_action()? {
                    Action::CreateTrial => {
                        let obs = Observation::new(
                            self.next_obs_id.fetch_and_increment(),
                            self.next_trial_id.fetch_and_increment(),
                        );
                        self.output.write(Event::Trial(TrialEvent::Started {
                            trial_id: obs.trial_id,
                            elapsed: self.start_time.elapsed(),
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
                        self.output.write(Event::Trial(TrialEvent::Finished {
                            trial_id,
                            elapsed: self.start_time.elapsed(),
                        }))?;
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
                    let trial_finished = self.optimizer.tell(&obs)?;

                    let obs_id = obs.id;
                    let trial_id = obs.trial_id;
                    let elapsed = self.start_time.elapsed();
                    self.output
                        .write(Event::Obs(ObservationEvent::Finished { obs, elapsed }))?;
                    self.obs_temp_dirs.remove(&obs_id);
                    if trial_finished {
                        self.output
                            .write(Event::Trial(TrialEvent::Finished { trial_id, elapsed }))?;
                        self.trial_temp_dirs.remove(&trial_id);
                    }
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

    fn is_study_finished(&self) -> bool {
        self.opt
            .runs
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
        let value = self.optimizer.ask(obs, &req.param_name, &req.param_type)?;
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
