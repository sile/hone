use crate::config::Config;
use crate::study::StudyServer;
use crate::{Error, ErrorKind, Result};
use std::env;
use std::mem;
use std::process::Command;
use structopt::StructOpt;

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
        let current_dir = track!(env::current_dir().map_err(Error::from))?;
        let study_name = if let Some(name) = &opt.study {
            name.clone()
        } else {
            track_assert_some!(
                current_dir.file_name().and_then(|n| n.to_str()),
                ErrorKind::InvalidInput
            )
            .to_string()
        };
        Ok(Self {
            opt,
            config,
            study_name,
        })
    }

    pub fn run(self) -> Result<()> {
        eprintln!("Study Name: {}", self.study_name);

        let study = track!(StudyServer::new(self.study_name.clone()))?;
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
