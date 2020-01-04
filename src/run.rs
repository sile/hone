use crate::config::Config;
use crate::optimizer::RandomOptimizer;
use crate::pubsub::PubSub;
use crate::study::{StudyServer, StudyServerHandle, TrialHandle};
use crate::{Error, Result};
use std::process::{Child, Command};
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Debug, StructOpt)]
pub struct RunOpt {
    #[structopt(long)]
    pub study: Option<String>,

    #[structopt(long, default_value = "1")]
    pub repeats: usize,
    // timeout, search-space, parallelism
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug)]
pub struct Runner {
    opt: RunOpt,
    config: Config,

    study_name: String,
}

impl Runner {
    pub fn new(opt: RunOpt, config: Config) -> Result<Self> {
        let study_name = if let Some(name) = &opt.study {
            name.clone()
        } else {
            Uuid::new_v4().to_string()
        };
        Ok(Self {
            opt,
            config,
            study_name,
        })
    }

    pub fn run(mut self) -> Result<()> {
        eprintln!("[HONE] Study Name: {}", self.study_name);
        let data_dir = track!(self.config.data_dir())?;
        let pubsub = PubSub::new(data_dir);
        let study = track!(StudyServer::new(
            self.study_name.clone(),
            pubsub,
            RandomOptimizer::new()
        ))?;
        let server_addr = track!(study.addr())?;
        eprintln!("[HONE] Server Address: {}", server_addr);

        let handle = study.spawn();

        for _ in 0..self.opt.repeats {
            track!(self.run_once(&handle))?;
        }
        Ok(())
    }

    fn run_once(&mut self, handle: &StudyServerHandle) -> Result<()> {
        let trial = track!(handle.start_trial())?;
        let child = track!(Command::new(&self.opt.command)
            .args(&self.opt.args)
            .spawn()
            .map_err(Error::from))?;
        eprintln!("[HONE] Spawn child process(pid={})", child.id());
        let mut trial = Trial::new(child, trial);
        let status = track!(trial.child.wait().map_err(Error::from))?;
        eprintln!("[HONE] Child process finished: {:?}", status);

        Ok(())
    }
}

#[derive(Debug)]
struct Trial {
    child: Child,
    handle: TrialHandle,
}

impl Trial {
    fn new(child: Child, handle: TrialHandle) -> Self {
        Self { child, handle }
    }
}

impl Drop for Trial {
    fn drop(&mut self) {
        if self.child.kill().is_ok() {
            let _ = self.child.wait(); // for preventing the child process becomes a zombie.
        }
    }
}
