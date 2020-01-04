use crate::metric::Metric;
use crate::param::{Param, ParamValue};
use crate::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufRead as _, BufReader, BufWriter, Seek as _, SeekFrom, Write as _};
use std::num::NonZeroU64;
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug)]
pub struct PubSub {
    data_dir: PathBuf,
}

impl PubSub {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    pub fn subscribe(&mut self, study_name: &str) -> Result<Subscriber> {
        let dir = self.data_dir.join(format!("{}/", study_name));
        if !dir.exists() {
            track_panic!(ErrorKind::InvalidInput, "No such study: {:?}", study_name);
        }

        Ok(Subscriber {
            dir,
            journals: HashMap::new(),
        })
    }

    pub fn channel(&mut self, study_name: &str) -> Result<PubSubChannel> {
        let dir = self.data_dir.join(format!("{}/", study_name));
        track!(fs::create_dir_all(&dir).map_err(Error::from); dir)?;

        let thread_id = Uuid::new_v4();
        let my_journal = track!(OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(dir.join(thread_id.to_string()))
            .map_err(Error::from))?;
        let subscriber = track!(self.subscribe(study_name))?;
        Ok(PubSubChannel {
            my_journal: JournalWriter::new(my_journal),
            subscriber,
        })
    }
}

// TODO: s/../Publisher/
#[derive(Debug)]
pub struct PubSubChannel {
    my_journal: JournalWriter,
    subscriber: Subscriber,
}

impl PubSubChannel {
    pub fn publish(&mut self, event: TrialAction) -> Result<()> {
        track!(self.my_journal.write_action(event))?;
        Ok(())
    }

    pub fn poll(&mut self) -> Result<Vec<(Uuid, TrialAction)>> {
        track!(self.subscriber.poll())
    }
}

#[derive(Debug)]
pub struct Subscriber {
    dir: PathBuf,
    journals: HashMap<PathBuf, JournalReader>,
}

impl Subscriber {
    pub fn poll(&mut self) -> Result<Vec<(Uuid, TrialAction)>> {
        let mut queue = Vec::new();
        for entry in track!(fs::read_dir(&self.dir).map_err(Error::from))? {
            let entry = track!(entry.map_err(Error::from))?;
            if !track!(entry.file_type().map_err(Error::from))?.is_file() {
                continue;
            }

            let path = entry.path();
            if !self.journals.contains_key(&path) {
                eprintln!("[HONE] New journal file: {:?}", path);
                let journal = track!(File::open(&path).map_err(Error::from))?;
                self.journals
                    .insert(path.clone(), JournalReader::new(journal));
            }
            let journal = self
                .journals
                .get_mut(&path)
                .unwrap_or_else(|| unreachable!());

            while let Some(action) = track!(journal.read_action())? {
                queue.push(action);
            }
        }

        Ok(queue)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrialAction {
    Start { id: Uuid, timestamp: Duration },
    Define { param: Param },
    Sample { name: String, value: ParamValue },
    Report { step: NonZeroU64, metric: Metric },
    End,
}

#[derive(Debug)]
struct JournalWriter {
    file: BufWriter<File>,
}

impl JournalWriter {
    fn new(file: File) -> Self {
        Self {
            file: BufWriter::new(file),
        }
    }

    fn write_action(&mut self, action: TrialAction) -> Result<()> {
        track!(serde_json::to_writer(&mut self.file, &action).map_err(Error::from))?;
        track!(self.file.write(&[b'\n'][..]).map_err(Error::from))?;
        track!(self.file.flush().map_err(Error::from))?;
        Ok(())
    }
}

#[derive(Debug)]
struct JournalReader {
    file: BufReader<File>,
    current_trial_id: Option<Uuid>,
}

impl JournalReader {
    fn new(file: File) -> Self {
        Self {
            file: BufReader::new(file),
            current_trial_id: None,
        }
    }

    fn read_action(&mut self) -> Result<Option<(Uuid, TrialAction)>> {
        // TODO: check mtime

        let position = track!(self.current_position())?;
        let mut line = String::new();

        let size = track!(self.file.read_line(&mut line).map_err(Error::from))?;
        if size == 0 {
            track!(self.seek(position))?;
            return Ok(None);
        }
        if line.as_bytes()[size - 1] != b'\n' {
            track!(self.seek(position))?;
            return Ok(None);
        }

        let action: TrialAction = track!(serde_json::from_str(&line).map_err(Error::from))?;
        let trial_id;
        match action {
            TrialAction::Start { id, .. } => {
                trial_id = id;
                self.current_trial_id = Some(id);
            }
            TrialAction::End => {
                trial_id =
                    track_assert_some!(self.current_trial_id.take(), ErrorKind::InvalidInput);
            }
            _ => {
                trial_id = track_assert_some!(
                    self.current_trial_id.clone().take(),
                    ErrorKind::InvalidInput
                );
            }
        }
        Ok(Some((trial_id, action)))
    }

    fn seek(&mut self, position: u64) -> Result<()> {
        track!(self
            .file
            .seek(SeekFrom::Start(position))
            .map_err(Error::from))?;
        Ok(())
    }

    fn current_position(&mut self) -> Result<u64> {
        track!(self.file.seek(SeekFrom::Current(0)).map_err(Error::from))
    }
}
