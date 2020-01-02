use crate::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_dir: Option<PathBuf>,
    // default_sampler, default_pruner
}

impl Default for Config {
    fn default() -> Self {
        Self { data_dir: None }
    }
}

impl Config {
    pub const FILE_NAME: &'static str = "config.json";
    pub const FILE_PATH: &'static str = ".hone/config.json";

    pub fn lookup_path() -> Result<PathBuf> {
        let current_dir = track!(env::current_dir().map_err(Error::from))?;
        let mut dir = current_dir.as_path();
        loop {
            let path = dir.join(Self::FILE_PATH);
            if path.exists() {
                return Ok(path);
            }

            dir = track_assert_some!(dir.parent(), ErrorKind::InvalidInput);
        }
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
