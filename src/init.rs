use crate::config::Config;
use crate::{Error, ErrorKind, Result};
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct InitOpt {
    #[structopt(long, default_value = ".hone/")]
    pub root_dir: PathBuf,
}

#[derive(Debug)]
pub struct Initializer {
    opt: InitOpt,
}

impl Initializer {
    pub fn new(opt: InitOpt) -> Self {
        Self { opt }
    }

    pub fn init(&self) -> Result<()> {
        track_assert!(
            !self.opt.root_dir.exists(),
            ErrorKind::InvalidInput;
            self.opt.root_dir
        );

        track!(fs::create_dir_all(&self.opt.root_dir).map_err(Error::from))?;
        track!(fs::create_dir_all(self.opt.root_dir.join("cache")).map_err(Error::from))?;
        track!(fs::create_dir_all(self.opt.root_dir.join("observations")).map_err(Error::from))?;

        let default_config = Config::default();
        track!(default_config.save_to_file(self.opt.root_dir.join("config.json")))?;

        println!("Initialized Hone directory in {:?}", self.opt.root_dir);
        Ok(())
    }
}
