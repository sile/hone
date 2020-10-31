#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    Study(StudyEvent),
    Trial(TrialEvent),
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
    Started { trial_id: usize, run_id: usize },
    Resumed { trial_id: usize, run_id: usize },
    Asked { run_id: usize },
    Told { run_id: usize },
    Finished { trial_id: usize },
}
