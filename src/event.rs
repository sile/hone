use crate::runner::StudyRunnerOpt;
use crate::trial::{Observation, ObservationId, TrialId};
use std::io::{BufRead, Write};
use std::time::Duration;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    Study(StudyEvent),
    Trial(TrialEvent),
    Obs(ObservationEvent),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StudyEvent {
    Started {
        #[serde(flatten)]
        opt: StudyRunnerOpt,
    },
    Resumed {
        #[serde(flatten)]
        opt: StudyRunnerOpt,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrialEvent {
    Started {
        trial_id: TrialId,
        // TODO: study_instance(?)
        elapsed: Duration, // TODO: ElapsedSeconds
    },
    Finished {
        trial_id: TrialId,
        elapsed: Duration,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationEvent {
    Started {
        obs_id: ObservationId,
        trial_id: TrialId,
        elapsed: Duration,
    },
    Finished {
        #[serde(flatten)]
        obs: Observation,
        elapsed: Duration,
    },
}

#[derive(Debug)]
pub struct EventWriter<W> {
    writer: W,
    file: Option<std::fs::File>,
}

impl<W: Write> EventWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer, file: None }
    }

    pub fn write(&mut self, event: Event) -> anyhow::Result<()> {
        serde_json::to_writer(&mut self.writer, &event)?;
        writeln!(&mut self.writer)?;
        if let Some(mut f) = self.file.as_mut() {
            serde_json::to_writer(&mut f, &event)?;
            writeln!(&mut f)?;
        }
        Ok(())
    }

    // TODO: remove
    pub fn add_file<P: AsRef<std::path::Path>>(&mut self, path: P) -> anyhow::Result<()> {
        self.file = Some(std::fs::File::create(path)?);
        Ok(())
    }
}

#[derive(Debug)]
pub struct EventReader<R> {
    reader: R,
}

impl<R: BufRead> EventReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn read(&mut self) -> anyhow::Result<Option<Event>> {
        let mut buf = String::new();
        let size = self.reader.read_line(&mut buf)?;
        if size == 0 {
            return Ok(None);
        }
        let event = serde_json::from_str(&buf)?;
        Ok(Some(event))
    }
}
