use crate::config::Config;
use crate::{Error, Result};
use serde_json;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, Write as _};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
// use uuid::Uuid;

const OBS_DIR: &str = ".hone/observations/";

#[derive(Debug, StructOpt)]
pub struct ObserveOpt {
    // paththrough: bool
// show: bool (print params and last value)
}

#[derive(Debug)]
pub struct Observer<R> {
    opt: ObserveOpt,
    config: Config,
    source: R,
}

impl<R: BufRead> Observer<R> {
    pub fn new(source: R, config: Config, opt: ObserveOpt) -> Self {
        Self {
            source,
            config,
            opt,
        }
    }

    pub fn observe(&mut self) -> Result<()> {
        // TODO: Support pruning

        let mut obs = track!(Observation::new(OBS_DIR, &self.config))?;

        let mut line = String::new();
        while 0 != track!(self.source.read_line(&mut line).map_err(Error::from))? {
            print!("{}", line);
            if line.starts_with("HONE:") {
                // TODO: support multiple-values
                let value: f64 = track!(line
                    .trim()
                    .split_at("HONE:".len())
                    .1
                    .parse()
                    .map_err(Error::from))?;
                track!(obs.record(&[value]))?;
            }
            line.clear();
        }

        track!(obs.finish())?;

        Ok(())
    }
}

#[derive(Debug)]
struct Observation {
    working_dir: PathBuf,
    file: File, // TODO: bufwriter?
}

impl Observation {
    pub fn new<P: AsRef<Path>>(path: P, config: &Config) -> Result<Self> {
        // let working_dir = path.as_ref().join(config.thread_id()).join("current/");
        // track!(fs::create_dir_all(&working_dir).map_err(Error::from))?;

        // let file = track!(OpenOptions::new()
        //     .write(true)
        //     .create_new(true)
        //     .open(working_dir.join("values.json"))
        //     .map_err(Error::from))?;
        // Ok(Self { file, working_dir })
        panic!()
    }

    pub fn record(&mut self, values: &[f64]) -> Result<()> {
        track!(serde_json::to_writer(&mut self.file, values).map_err(Error::from))?;
        track!(writeln!(&mut self.file).map_err(Error::from))?;
        Ok(())
    }

    pub fn finish(self) -> Result<()> {
        // TODO
        Ok(())
    }
}

impl Drop for Observation {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.working_dir);
    }
}
