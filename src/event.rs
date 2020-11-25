use crate::study::StudySpec;
use crate::trial::{Observation, ObservationId, TrialId};
use std::io::{BufRead, Write};
use std::time::Duration;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    Study(StudyEvent),
    Trial(TrialEvent),
    Observation(ObservationEvent),
}

impl Event {
    pub fn elapsed(&self) -> Option<Duration> {
        match self {
            Self::Observation(ObservationEvent::Started { elapsed, .. })
            | Self::Observation(ObservationEvent::Finished { elapsed, .. }) => Some(*elapsed),
            _ => None,
        }
    }

    pub fn study_started() -> Event {
        Self::Study(StudyEvent::Started)
    }

    pub fn study_defined(spec: StudySpec) -> Event {
        Self::Study(StudyEvent::Defined { spec })
    }

    pub fn trial_started(trial_id: TrialId) -> Event {
        Self::Trial(TrialEvent::Started { trial_id })
    }

    pub fn trial_finished(trial_id: TrialId) -> Event {
        Self::Trial(TrialEvent::Finished { trial_id })
    }

    pub fn observation_started(
        obs_id: ObservationId,
        trial_id: TrialId,
        elapsed: Duration,
    ) -> Event {
        Self::Observation(ObservationEvent::Started {
            obs_id,
            trial_id,
            elapsed,
        })
    }

    pub fn observation_finished(obs: Observation, elapsed: Duration) -> Event {
        Self::Observation(ObservationEvent::Finished { obs, elapsed })
    }
}

#[derive(Debug)]
pub enum EventOrLine {
    Event(Event),
    Line(String, anyhow::Error),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StudyEvent {
    Started,
    Defined {
        #[serde(flatten)]
        spec: StudySpec,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrialEvent {
    Started { trial_id: TrialId },
    Finished { trial_id: TrialId },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationEvent {
    Started {
        obs_id: ObservationId,
        trial_id: TrialId,
        elapsed: Duration,
    },
    // TODO: Queued
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

    // TODO: flush
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

    pub fn read_event_or_line(&mut self) -> anyhow::Result<Option<EventOrLine>> {
        let mut buf = String::new();
        let size = self.reader.read_line(&mut buf)?;
        if size == 0 {
            return Ok(None);
        }
        match serde_json::from_str(&buf) {
            Ok(event) => Ok(Some(EventOrLine::Event(event))),
            Err(err) => Ok(Some(EventOrLine::Line(buf, err.into()))),
        }
    }
}
