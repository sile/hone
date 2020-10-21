use crate::envvar;
use crate::rpc;
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
pub struct Runner {
    options: RunOpt,
    server_addr: SocketAddr,
    channel: rpc::Channel,
}

impl Runner {
    pub fn new(options: RunOpt) -> anyhow::Result<Self> {
        let (server_addr, channel) = rpc::spawn_rpc_server()?;
        eprintln!("SERVER_ADDR: {}", server_addr);
        Ok(Self {
            options,
            server_addr,
            channel,
        })
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        let mut optimizer = crate::optimizer::RandomOptimizer::new(crate::rng::ArcRng::new(0));
        let mut workers: Vec<Child> = Vec::new();
        let mut trial_id = 0;
        while trial_id < self.options.repeat {
            if let Some(m) = self.channel.try_recv() {
                eprintln!("RECV: {:?}", m);
                continue;
            }

            while workers.len() < self.options.parallelism.get() {
                workers.push(self.spawn_worker(trial_id)?);
                eprintln!("New worker started: trial={}", trial_id);
                trial_id += 1;
            }

            let mut i = 0;
            while i < workers.len() {
                match workers[i].try_wait() {
                    Err(e) => todo!("{}", e),
                    Ok(None) => {
                        i += 1;
                    }
                    Ok(status) => {
                        // TODO: tell
                        eprintln!("Worker finished: {:?}", status);
                        workers.swap_remove(i);
                    }
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        Ok(())
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
