use crate::trial::{ObservationId, TrialId};
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

#[derive(Debug)]
pub struct TempDirs {
    study: Option<TempDir>,
    trials: HashMap<TrialId, TempDir>,
    observations: HashMap<ObservationId, TempDir>,
}

impl TempDirs {
    pub fn new() -> Self {
        Self {
            study: None,
            trials: HashMap::new(),
            observations: HashMap::new(),
        }
    }

    pub fn create_study_tempdir(&mut self, parent: Option<&PathBuf>) -> anyhow::Result<PathBuf> {
        let (temp, path) = Self::ensure_temp_dir_created(self.study.as_ref(), parent)?;
        if let Some(temp) = temp {
            self.study = Some(temp);
        }
        Ok(path)
    }

    pub fn create_trial_tempdir(
        &mut self,
        id: TrialId,
        parent: Option<&PathBuf>,
    ) -> anyhow::Result<PathBuf> {
        let (temp, path) = Self::ensure_temp_dir_created(self.trials.get(&id), parent)?;
        if let Some(temp) = temp {
            self.trials.insert(id, temp);
        }
        Ok(path)
    }

    pub fn create_obs_tempdir(
        &mut self,
        id: ObservationId,
        parent: Option<&PathBuf>,
    ) -> anyhow::Result<PathBuf> {
        let (temp, path) = Self::ensure_temp_dir_created(self.observations.get(&id), parent)?;
        if let Some(temp) = temp {
            self.observations.insert(id, temp);
        }
        Ok(path)
    }

    pub fn remove_trial_tempdir(&mut self, id: TrialId) {
        self.trials.remove(&id);
    }

    pub fn remove_obs_tempdir(&mut self, id: ObservationId) {
        self.observations.remove(&id);
    }

    fn ensure_temp_dir_created(
        tempdir: Option<&tempfile::TempDir>,
        parent: Option<&PathBuf>,
    ) -> anyhow::Result<(Option<TempDir>, PathBuf)> {
        if let Some(temp) = tempdir {
            Ok((None, temp.path().to_path_buf()))
        } else if let Some(parent) = parent {
            std::fs::create_dir_all(parent)?;
            let temp = TempDir::new_in(parent)?;
            let path = temp.path().to_path_buf();
            Ok((Some(temp), path))
        } else {
            let temp = TempDir::new()?;
            let path = temp.path().to_path_buf();
            Ok((Some(temp), path))
        }
    }
}
