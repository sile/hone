use crate::envvar;
use crate::event::EventWriter;
use crate::metric::MetricInstance;
use crate::optimizer::Action;
use crate::optimizer::{Optimize, Optimizer};
use crate::param::{ParamInstance, ParamValue};
use crate::rpc;
use crate::trial::{Observation, ObservationId, TrialId};
use anyhow::Context;
use std::collections::HashMap;
use std::io::Write;
use std::num::NonZeroUsize;
use std::process::{Child, Command, ExitStatus};
use std::time::Duration;

#[derive(Debug)]
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
        // TODO: capture stdout/stderr
        let proc = Command::new(&self.path)
            .args(&self.args)
            .env(envvar::KEY_SERVER_ADDR, rpc_server_addr.to_string())
            .env(envvar::KEY_OBSERVATION_ID, observation_id.get().to_string())
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

#[derive(Debug)]
pub struct StudyRunnerOpt {
    // timeout: {study,trial,observation,observation}
    // tempdir: {study,trial,observation,observation}
    pub study_name: String,
    pub workers: NonZeroUsize,
    pub runs: Option<usize>,
    pub command: CommandRunnerOpt,
}

#[derive(Debug)]
pub struct StudyRunner<W> {
    output: EventWriter<W>,
    observationnings: Vec<CommandRunner>,
    observations: HashMap<ObservationId, Observation>,
    finished_observations: usize,
    next_observation_id: ObservationId,
    next_obs_id: ObservationId,
    next_trial_id: TrialId,
    rpc_server_addr: std::net::SocketAddr,
    rpc_channel: rpc::Channel,
    optimizer: Optimizer,
    opt: StudyRunnerOpt,
}

impl<W: Write> StudyRunner<W> {
    pub fn new(output: W, optimizer: Optimizer, opt: StudyRunnerOpt) -> anyhow::Result<Self> {
        let (rpc_server_addr, rpc_channel) = rpc::spawn_rpc_server()?;
        eprintln!("[HONE] RPC server: {}", rpc_server_addr);

        Ok(Self {
            output: EventWriter::new(output),
            observationnings: Vec::new(),
            observations: HashMap::new(),
            finished_observations: 0,
            rpc_server_addr,
            rpc_channel,
            next_observation_id: ObservationId::new(0),
            next_obs_id: ObservationId::new(0),
            next_trial_id: TrialId::new(0),
            optimizer,
            opt,
        })
    }

    fn start_observation(&mut self, obs: Observation) -> anyhow::Result<()> {
        let observation_id = self.next_observation_id.fetch_and_increment();
        self.observationnings.push(
            self.opt
                .command
                .spawn(observation_id, self.rpc_server_addr)?,
        );
        self.observations.insert(observation_id, obs);
        eprintln!("[HONE] Spawn new process.");
        Ok(())
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        let mut did_nothing;

        while !self.is_study_finished() {
            did_nothing = true;

            while self.observationnings.len() < self.opt.workers.get() {
                match self.optimizer.next_action()? {
                    Action::CreateTrial => {
                        let obs = Observation::new(
                            self.next_obs_id.fetch_and_increment(),
                            self.next_trial_id.fetch_and_increment(),
                        );
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
                    Action::FinishTrial { .. } => {
                        todo!();
                    }
                    Action::WaitObservations => {
                        break;
                    }
                    Action::QuitOptimization => {
                        // TODO: kill observationning processes
                        return Ok(());
                    }
                }
            }

            while let Some(message) = self.rpc_channel.try_recv() {
                self.handle_message(message)?;
                did_nothing = false;
            }

            let mut i = 0;
            while i < self.observationnings.len() {
                if let Some(status) = self.observationnings[i].try_wait()? {
                    eprintln!("[HONE] Process exited: {}", status);
                    let finished = self.observationnings.swap_remove(i);
                    self.finished_observations += 1;
                    let obs = self
                        .observations
                        .remove(&finished.observation_id)
                        .expect("bug");
                    self.optimizer.tell(&obs)?;
                    // TODO: retry if failed
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
        }
        Ok(())
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
        // TODO: record event
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
