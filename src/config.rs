use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::env;
use std::fs::File;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    // TODO: delete or rename to "default_thread_id" ?
    pub id: Uuid,
    // TODO: shared_dir, local_dir
}

impl Config {
    pub fn thread_id(&self) -> String {
        env::var("HONE_THREAD_ID").unwrap_or_else(|_| self.id.to_string().clone())
    }

    pub fn load_from_default_file() -> Result<Self> {
        track!(Self::load_from_file(".hone/config.json"))
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = track!(File::open(path).map_err(Error::from))?;
        let config = track!(serde_json::from_reader(file).map_err(Error::from))?;
        Ok(config)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file = track!(File::create(path).map_err(Error::from))?;
        track!(serde_json::to_writer_pretty(file, self).map_err(Error::from))?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self { id: Uuid::new_v4() }
    }
}
