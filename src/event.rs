#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    Study(StudyEvent),
    Trial(TrialEvent),
    Run(RunEvent),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StudyEvent {
    Started { name: String },
    Resumed { rename: Option<String> },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrialEvent {
    Started { trial_id: usize },
    Finished { trial_id: usize },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObsEvent {
    Started { obs_id: usize, trial_id: usize },
    Asked { obs_id: usize },
    Told { obs_id: usize },
    Finished { obs_id: usize },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunEvent {
    Started { run_id: usize, obs_id: usize },
    Finished { run_id: usize, exit_code: usize },
}
