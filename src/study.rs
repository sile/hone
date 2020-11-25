use crate::tuners::TunerSpec;
use std::collections::BTreeMap;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StudySpec {
    pub name: String,
    pub id: Uuid,
    pub attrs: BTreeMap<String, String>,
    pub tuner: TunerSpec,
    pub command: CommandSpec,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommandSpec {
    pub path: PathBuf,
    pub args: Vec<String>,
}
