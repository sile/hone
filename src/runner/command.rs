use crate::envvar;
use crate::study::StudySpec;
use crate::trial::Observation;
use anyhow::Context;
use std::process::{Child, Command, Stdio};

#[derive(Debug)]
pub struct CommandRunner {
    obs: Observation,
    proc: Child,
}

impl CommandRunner {
    pub fn spawn(
        study: &StudySpec,
        obs: Observation,
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
        Ok(CommandRunner { obs, proc })
    }

    pub fn obs(&self) -> &Observation {
        &self.obs
    }

    pub fn obs_mut(&mut self) -> &mut Observation {
        &mut self.obs
    }

    pub fn into_obs(self) -> Observation {
        self.obs
    }

    pub fn is_exited(&mut self) -> anyhow::Result<bool> {
        if let Some(exit_status) = self.proc.try_wait()? {
            self.obs.exit_status = exit_status.code();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn kill(&mut self) -> anyhow::Result<()> {
        self.proc.kill()?;
        Ok(())
    }
}
