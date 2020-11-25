use crate::study::StudySpec;
use crate::trial::{Observation, ObservationId, TrialId};
use crate::types::ElapsedSeconds;
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
            | Self::Observation(ObservationEvent::Finished { elapsed, .. }) => {
                Some(elapsed.to_duration())
            }
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
            elapsed: elapsed.into(),
        })
    }

    pub fn observation_finished(obs: Observation, elapsed: Duration) -> Event {
        Self::Observation(ObservationEvent::Finished {
            obs,
            elapsed: elapsed.into(),
        })
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
        elapsed: ElapsedSeconds,
    },
    // TODO: Queued
    Finished {
        #[serde(flatten)]
        obs: Observation,
        elapsed: ElapsedSeconds,
    },
}

#[derive(Debug)]
pub struct EventWriter<W> {
    writer: W,
}

impl<W: Write> EventWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn write(&mut self, event: Event) -> anyhow::Result<()> {
        serde_json::to_writer(&mut self.writer, &event)?;
        writeln!(&mut self.writer)?;
        self.writer.flush()?;
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
