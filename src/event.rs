use std::io::Write;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    Study(StudyEvent),
    Trial(TrialEvent),
    Observation(ObservationEvent),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StudyEvent {
    Started { name: String },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrialEvent {
    Started { trial_id: usize },
    Finished { trial_id: usize },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationEvent {
    Started { obs_id: usize, trial_id: usize },
    Finished { obs_id: usize },
}

#[derive(Debug)]
pub struct EventWriter<W> {
    writer: W,
}

impl<W: Write> EventWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}
