use crate::envvar;
use crate::optimizer;
use crate::optimizer::Optimizer;
use crate::rpc;
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::num::NonZeroUsize;
use std::process::{Child, Command};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct RunOpt {
    #[structopt(long, default_value = "1")]
    pub repeat: usize,

    // TODO: Implement
    #[structopt(long)]
    pub tempdir: bool,

    #[structopt(long, default_value = "1")]
    pub parallelism: NonZeroUsize,

    // TODO: seed
    // TODO: timeout, search-space, retry, sync
    pub command: String,
    pub args: Vec<String>,
}

impl RunOpt {
    pub fn run(self) -> anyhow::Result<()> {
        Runner::new(self)?.run()
    }
}

#[derive(Debug)]
pub struct TrialState {
    trial_id: u64,
    params: BTreeMap<String, crate::hp::HpValue>,
    values: BTreeMap<String, f64>,
    failed: bool,
}

impl TrialState {
    pub fn new(trial_id: u64) -> Self {
        Self {
            trial_id,
            params: BTreeMap::new(),
            values: BTreeMap::new(),
            failed: false,
        }
    }

    pub fn to_evaluated_trial(
        &self,
        ss: &optimizer::SearchSpace,
        os: &optimizer::ObjectiveSpace,
    ) -> anyhow::Result<optimizer::EvaluatedTrial> {
        let mut params = Vec::new();
        for (name, distribution) in ss.params.iter() {
            if let Some(v) = self.params.get(name) {
                params.push(distribution.warp(v)?);
            } else {
                params.push(std::f64::NAN);
            }
        }

        let mut values = Vec::new();
        for (name, domain) in os.values.iter() {
            if let Some(v) = self.values.get(name) {
                params.push(if domain.minimize { *v } else { -*v });
            } else {
                values.push(std::f64::NAN);
            }
        }

        Ok(optimizer::EvaluatedTrial {
            trial_id: self.trial_id,
            params,
            values: if self.failed { None } else { Some(values) },
        })
    }
}

#[derive(Debug)]
pub struct Runner {
    options: RunOpt,
    server_addr: SocketAddr,
    channel: rpc::Channel,
    optimizer: crate::optimizer::RandomOptimizer,
    search_space: crate::optimizer::SearchSpace,
    objective_space: crate::optimizer::ObjectiveSpace,
    running_trials: BTreeMap<u64, TrialState>,
    evaluated_trials: Vec<TrialState>,
}

impl Runner {
    pub fn new(options: RunOpt) -> anyhow::Result<Self> {
        let (server_addr, channel) = rpc::spawn_rpc_server()?;
        eprintln!("SERVER_ADDR: {}", server_addr);
        Ok(Self {
            options,
            server_addr,
            channel,
            optimizer: optimizer::RandomOptimizer::new(crate::rng::ArcRng::new(0)),
            search_space: optimizer::SearchSpace::new(),
            objective_space: optimizer::ObjectiveSpace::new(),
            running_trials: BTreeMap::new(),
            evaluated_trials: Vec::new(),
        })
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        let mut workers: Vec<(u64, Child)> = Vec::new();
        let mut trial_id = 0;
        while self.evaluated_trials.len() < self.options.repeat {
            if let Some(m) = self.channel.try_recv() {
                self.handle_message(m)?;
            }

            while workers.len() < self.options.parallelism.get() {
                workers.push((trial_id as u64, self.spawn_worker(trial_id)?));
                eprintln!("New worker started: trial={}", trial_id);
                self.running_trials
                    .insert(trial_id as u64, TrialState::new(trial_id as u64));
                trial_id += 1;
            }

            let mut i = 0;
            while i < workers.len() {
                match workers[i].1.try_wait() {
                    Err(e) => todo!("{}", e),
                    Ok(None) => {
                        i += 1;
                    }
                    Ok(status) => {
                        // TODO: tell
                        eprintln!("Worker finished: {:?}", status);
                        let (trial_id, _) = workers.swap_remove(i);
                        let mut trial = self.running_trials.remove(&trial_id).expect("unreachable");
                        trial.failed = !status.map_or(true, |s| s.success());
                        self.evaluated_trials.push(trial);
                    }
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        Ok(())
    }

    fn handle_message(&mut self, m: rpc::Message) -> anyhow::Result<()> {
        match m {
            rpc::Message::Ask { req, reply } => {
                let value = self.handle_ask(req)?;
                let _ = reply.send(value);
            }
            rpc::Message::Tell { req, reply } => {
                let result = self.handle_tell(req)?;
                let _ = reply.send(result);
            }
        }
        Ok(())
    }

    fn handle_tell(&mut self, req: rpc::TellReq) -> anyhow::Result<Result<(), rpc::TellError>> {
        todo!()
    }

    fn handle_ask(
        &mut self,
        req: rpc::AskReq,
    ) -> anyhow::Result<Result<crate::hp::HpValue, rpc::AskError>> {
        let trial = if let Some(trial) = self.running_trials.get_mut(&req.trial_id) {
            trial
        } else {
            return Ok(Err(rpc::AskError::InvalidRequest));
        };
        if let Some(value) = trial.params.get(&req.param_name) {
            if req.distribution.is_some() {
                // TODO: Error if req.distribution doesn't contain the value.
            }
            return Ok(Ok(value.clone()));
        } else if req.distribution.is_none() {
            return Ok(Err(rpc::AskError::InvalidRequest));
        }

        let hp_distribution = req.distribution.expect("unreachable");
        let is_expanded = self
            .search_space
            .expand_if_need(&req.param_name, &hp_distribution)?;
        if is_expanded {
            self.optimizer
                .initialize(&self.search_space, &self.objective_space)?;
            for t in &self.evaluated_trials {
                self.optimizer
                    .tell(&t.to_evaluated_trial(&self.search_space, &self.objective_space)?)?;
            }
        }

        let distribution = crate::optimizer::Distribution::from(hp_distribution);

        let param_index = self
            .search_space
            .index(&req.param_name)
            .expect("unreachable");
        let param_value = self
            .optimizer
            .ask(req.trial_id, param_index, distribution)?;
        let param_value = self.search_space.unwarp(&req.param_name, param_value)?;
        trial
            .params
            .insert(req.param_name.clone(), param_value.clone());

        Ok(Ok(param_value))
    }

    fn spawn_worker(&mut self, trial_id: usize) -> anyhow::Result<Child> {
        let child = Command::new(&self.options.command)
            .args(&self.options.args)
            .env(envvar::KEY_SERVER_ADDR, self.server_addr.to_string())
            .env(envvar::KEY_TRIAL_ID, trial_id.to_string())
            .spawn()?;
        eprintln!("[HONE] Spawn child process(pid={})", child.id());
        Ok(child)
    }
}

// use crate::config::Config;
// use crate::optimizer::RandomOptimizer;
// use crate::pubsub::PubSub;
// use crate::study::{StudyServer, StudyServerHandle, TrialHandle};
// use crate::{Error, Result};
// use structopt::StructOpt;
// use uuid::Uuid;

// #[derive(Debug, StructOpt)]
// pub struct RunOpt {
//     #[structopt(long)]
//     pub study: Option<String>,

//     #[structopt(long, default_value = "1")]
//     pub repeats: usize,
//     // timeout, search-space, parallelism
//     pub command: String,
//     pub args: Vec<String>,
// }

// #[derive(Debug)]
// pub struct Runner {
//     opt: RunOpt,
//     config: Config,

//     study_name: String,
// }

// impl Runner {
//     pub fn new(opt: RunOpt, config: Config) -> Result<Self> {
//         let study_name = if let Some(name) = &opt.study {
//             name.clone()
//         } else {
//             Uuid::new_v4().to_string()
//         };
//         Ok(Self {
//             opt,
//             config,
//             study_name,
//         })
//     }

//     pub fn run(mut self) -> Result<()> {
//         eprintln!("[HONE] Study Name: {}", self.study_name);
//         let data_dir = track!(self.config.data_dir())?;
//         let pubsub = PubSub::new(data_dir);
//         let study = track!(StudyServer::new(
//             self.study_name.clone(),
//             pubsub,
//             RandomOptimizer::new()
//         ))?;
//         let server_addr = track!(study.addr())?;
//         eprintln!("[HONE] Server Address: {}", server_addr);

//         let handle = study.spawn();

//         for _ in 0..self.opt.repeats {
//             track!(self.run_once(&handle))?;
//         }
//         Ok(())
//     }

//     fn run_once(&mut self, handle: &StudyServerHandle) -> Result<()> {
//         let trial = track!(handle.start_trial())?;
//         let child = track!(Command::new(&self.opt.command)
//             .args(&self.opt.args)
//             .spawn()
//             .map_err(Error::from))?;
//         eprintln!("[HONE] Spawn child process(pid={})", child.id());
//         let mut trial = Trial::new(child, trial);
//         let status = track!(trial.child.wait().map_err(Error::from))?;
//         eprintln!("[HONE] Child process finished: {:?}", status);

//         Ok(())
//     }
// }

// #[derive(Debug)]
// struct Trial {
//     child: Child,
//     handle: TrialHandle,
// }

// impl Trial {
//     fn new(child: Child, handle: TrialHandle) -> Self {
//         Self { child, handle }
//     }
// }

// impl Drop for Trial {
//     fn drop(&mut self) {
//         if self.child.kill().is_ok() {
//             let _ = self.child.wait(); // for preventing the child process becomes a zombie.
//         }
//     }
// }
