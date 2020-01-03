use crate::config::Config;
use crate::pubsub::PubSub;
use crate::study::StudyServer;
use crate::{Error, Result};
use std::mem;
use std::process::Command;
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Debug, StructOpt)]
pub struct RunOpt {
    #[structopt(long)]
    pub study: Option<String>,
    // timeout, repeats, search-space
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

    pub fn run(self) -> Result<()> {
        eprintln!("Study Name: {}", self.study_name);

        let data_dir = track!(self.config.data_dir())?;
        let pubsub = PubSub::new(data_dir);
        let study = track!(StudyServer::new(self.study_name.clone(), pubsub))?;
        let server_addr = track!(study.addr())?;
        eprintln!("Server Address: {}", server_addr);

        let handle = study.spawn();

        let trial = track!(handle.start_trial())?;

        // TODO: kill child process when crached.
        let mut child = track!(Command::new(&self.opt.command)
            .args(&self.opt.args)
            .spawn()
            .map_err(Error::from))?;
        eprintln!("Spawn child process(pid={})", child.id());
        let status = track!(child.wait().map_err(Error::from))?;
        eprintln!("Child process finished: {:?}", status);
        mem::drop(trial);

        Ok(())
    }
}
