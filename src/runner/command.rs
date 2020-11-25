use crate::envvar;
use crate::study::StudySpec;
use crate::trial::{Observation, ObservationId};
use anyhow::Context;
use std::process::{Child, Command, ExitStatus, Stdio};

#[derive(Debug)]
pub struct CommandRunner {
    obs_id: ObservationId,
    proc: Child,
}

impl CommandRunner {
    pub fn spawn(
        study: &StudySpec,
        obs: &Observation,
        rpc_server_addr: std::net::SocketAddr,
    ) -> anyhow::Result<Self> {
        let mut command = Command::new(&study.command.path);
        command
            .args(&study.command.args)
            .env(envvar::KEY_SERVER_ADDR, rpc_server_addr.to_string())
            .env(envvar::KEY_STUDY_ID, study.id.to_string())
            .env(envvar::KEY_TRIAL_ID, obs.trial_id.get().to_string())
            .env(envvar::KEY_OBSERVATION_ID, obs.id.get().to_string())
            .stdin(Stdio::null());
        let proc = command
            .spawn()
            .with_context(|| format!("Failed to spawn command: {:?}", study.command.path))?;
        Ok(CommandRunner {
            obs_id: obs.id,
            proc,
        })
    }

    pub const fn obs_id(&self) -> ObservationId {
        self.obs_id
    }

    pub fn try_wait(&mut self) -> anyhow::Result<Option<ExitStatus>> {
        let exit_status = self.proc.try_wait()?;
        Ok(exit_status)
    }
}
