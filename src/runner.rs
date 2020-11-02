use crate::envvar;
use crate::event::EventWriter;
use crate::rpc;
use crate::trial::RunId;
use anyhow::Context;
use std::io::Write;
use std::num::NonZeroUsize;
use std::process::{Child, Command};
use std::time::Duration;

#[derive(Debug)]
pub struct CommandRunnerOpt {
    pub path: String,
    pub args: Vec<String>,
}

impl CommandRunnerOpt {
    pub fn spawn(
        &self,
        run_id: RunId,
        rpc_server_addr: std::net::SocketAddr,
    ) -> anyhow::Result<CommandRunner> {
        // TODO: capture stdout/stderr
        let proc = Command::new(&self.path)
            .args(&self.args)
            .env(envvar::KEY_SERVER_ADDR, rpc_server_addr.to_string())
            .env(envvar::KEY_RUN_ID, run_id.get().to_string())
            .spawn()
            .with_context(|| format!("Failed to spawn command: {:?}", self.path))?;
        Ok(CommandRunner { proc })
    }
}

#[derive(Debug)]
pub struct CommandRunner {
    proc: Child,
}

#[derive(Debug)]
pub struct StudyRunnerOpt {
    // timeout: {study,trial,observation,run}
    // tempdir: {study,trial,observation,run}
    pub study_name: String,
    pub workers: NonZeroUsize,
    pub runs: Option<usize>,
    pub command: CommandRunnerOpt,
}

#[derive(Debug)]
pub struct StudyRunner<W> {
    output: EventWriter<W>,
    runnings: Vec<CommandRunner>,
    finished_runs: usize,
    next_run_id: RunId,
    rpc_server_addr: std::net::SocketAddr,
    rpc_channel: rpc::Channel,
    opt: StudyRunnerOpt,
}

impl<W: Write> StudyRunner<W> {
    pub fn new(output: W, opt: StudyRunnerOpt) -> anyhow::Result<Self> {
        let (rpc_server_addr, rpc_channel) = rpc::spawn_rpc_server()?;
        eprintln!("[HONE] RPC server: {}", rpc_server_addr);

        Ok(Self {
            output: EventWriter::new(output),
            runnings: Vec::new(),
            finished_runs: 0,
            rpc_server_addr,
            rpc_channel,
            next_run_id: RunId::new(0),
            opt,
        })
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        let mut did_nothing;

        while !self.is_study_finished() {
            did_nothing = true;

            while self.runnings.len() < self.opt.workers.get() {
                eprintln!("[HONE] Spawn new process.");
                let run_id = self.next_run_id.fetch_and_increment();
                self.runnings
                    .push(self.opt.command.spawn(run_id, self.rpc_server_addr)?);
                did_nothing = false;
            }

            while let Some(message) = self.rpc_channel.try_recv() {
                eprintln!("[HONE] Recv: {:?}", message);
                did_nothing = false;
            }

            if did_nothing {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        Ok(())
    }

    pub fn is_study_finished(&self) -> bool {
        self.opt.runs.map_or(false, |n| self.finished_runs >= n)
    }
}
