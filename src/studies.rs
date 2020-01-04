use crate::config::Config;
use crate::{Error, ErrorKind, Result};
use std::fs;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct StudiesOpt {}

pub fn list_studies(_opt: StudiesOpt, config: &Config) -> Result<Vec<String>> {
    let mut names = Vec::new();

    let dir = track!(config.data_dir())?;
    for entry in track!(fs::read_dir(&dir).map_err(Error::from))? {
        let entry = track!(entry.map_err(Error::from))?;
        if !track!(entry.file_type().map_err(Error::from))?.is_dir() {
            continue;
        }

        let name = track_assert_some!(
            entry
                .path()
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.to_owned()),
            ErrorKind::InvalidInput
        );
        names.push(name);
    }

    Ok(names)
}
