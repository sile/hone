use crate::trial::TrialId;

pub const KEY_SERVER_ADDR: &str = "HONE_SERVER_ADDR";
pub const KEY_TRIAL_ID: &str = "HONE_TRIAL_ID";

pub fn get_trial_id() -> Result<TrialId, EnvVarError> {
    let value =
        std::env::var(KEY_TRIAL_ID).map_err(|e| EnvVarError::from_var_error(KEY_TRIAL_ID, e))?;
    let id = value
        .parse()
        .map_err(|e| EnvVarError::from_other_error(KEY_TRIAL_ID, e))?;
    Ok(TrialId::new(id))
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
