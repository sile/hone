use crate::config::Config;
use crate::{Error, ErrorKind, Result};
use std::fs;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct InitOpt {
    #[structopt(long)]
    pub data_dir: Option<PathBuf>,
}

impl InitOpt {}

#[derive(Debug)]
pub struct Initializer {
    opt: InitOpt,
}

impl Initializer {
    pub fn new(opt: InitOpt) -> Self {
        Self { opt }
    }

    pub fn init(&self) -> Result<()> {
        let root_dir = track_assert_some!(Path::new(Config::FILE_PATH).parent(), ErrorKind::Bug);
        track_assert!(
            !root_dir.exists(),
            ErrorKind::InvalidInput;
            root_dir
        );

        track!(fs::create_dir_all(&root_dir).map_err(Error::from))?;

        let mut config = Config::default();
        config.data_dir = self.opt.data_dir.clone();
        track!(config.save_to_file(root_dir.join(Config::FILE_NAME)))?;

        eprintln!("Initialized Hone directory in {:?}", root_dir);
        Ok(())
    }
}
