use crate::trial::{ObservationId, TrialId};
use std::net::SocketAddr;

pub const KEY_SERVER_ADDR: &str = "HONE_SERVER_ADDR";
pub const KEY_STUDY_INSTNACE_ID: &str = "HONE_STUDY_INSTANCE_ID";
pub const KEY_TRIAL_ID: &str = "HONE_TRIAL_ID";
pub const KEY_OBSERVATION_ID: &str = "HONE_OBS_ID";
pub const KEY_STUDY_DIR: &str = "HONE_STUDY_DIR";
pub const KEY_TRIAL_DIR: &str = "HONE_TRIAL_DIR";
pub const KEY_OBSERVATION_DIR: &str = "HONE_OBS_DIR";
pub const KEY_STUDY_TEMP_DIR: &str = "HONE_STUDY_TEMP_DIR";
pub const KEY_TRIAL_TEMP_DIR: &str = "HONE_TRIAL_TEMP_DIR";
pub const KEY_OBSERVATION_TEMP_DIR: &str = "HONE_OBS_TEMP_DIR";

pub fn get_server_addr() -> Result<SocketAddr, EnvVarError> {
    let value = std::env::var(KEY_SERVER_ADDR)
        .map_err(|e| EnvVarError::from_var_error(KEY_SERVER_ADDR, e))?;
    let server_addr: SocketAddr = value
        .parse()
        .map_err(|e| EnvVarError::from_other_error(KEY_SERVER_ADDR, e))?;
    Ok(server_addr)
}

pub fn get_string(key: &'static str) -> Result<String, EnvVarError> {
    let value = std::env::var(key).map_err(|e| EnvVarError::from_var_error(key, e))?;
    Ok(value)
}

pub fn get_trial_id() -> Result<TrialId, EnvVarError> {
    let value =
        std::env::var(KEY_TRIAL_ID).map_err(|e| EnvVarError::from_var_error(KEY_TRIAL_ID, e))?;
    let id = value
        .parse()
        .map_err(|e| EnvVarError::from_other_error(KEY_TRIAL_ID, e))?;
    Ok(TrialId::new(id))
}

pub fn get_observation_id() -> Result<ObservationId, EnvVarError> {
    let value = std::env::var(KEY_OBSERVATION_ID)
        .map_err(|e| EnvVarError::from_var_error(KEY_OBSERVATION_ID, e))?;
    let id = value
        .parse()
        .map_err(|e| EnvVarError::from_other_error(KEY_OBSERVATION_ID, e))?;
    Ok(ObservationId::new(id))
}

#[derive(Debug, thiserror::Error)]
pub enum EnvVarError {
    #[error("the environment variable {key:?} is not found")]
    NotFound { key: &'static str },

    #[error("the environment variable {key:?} contains an invalid value: {source}")]
    Other {
        key: &'static str,
        source: anyhow::Error,
    },
}

impl EnvVarError {
    fn from_var_error(key: &'static str, e: std::env::VarError) -> Self {
        match e {
            std::env::VarError::NotPresent => Self::NotFound { key },
            e => Self::Other {
                key,
                source: e.into(),
            },
        }
    }

    fn from_other_error<E>(key: &'static str, e: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Other {
            key,
            source: e.into(),
        }
    }
}
