use crate::attr::Attr;
use crate::event::EventReader;
use crate::runner::{StudyRunner, StudyRunnerOpt};
use crate::study::{CommandSpec, StudySpec};
use crate::tuners::TunerSpec;
use anyhow::Context;
use std::io::{BufReader, Write};
use std::num::NonZeroUsize;
use std::path::PathBuf;

#[derive(Debug, clap::Args)]
pub struct RunOpt {
    #[clap(long = "name")]
    pub study_name: Option<String>,

    #[clap(long)]
    pub study_attrs: Vec<Attr>,

    #[clap(long, default_value = "1")]
    pub workers: NonZeroUsize,

    #[clap(long, short = 'n')]
    pub repeat: Option<usize>,

    #[clap(long)]
    pub load: Vec<PathBuf>,

    #[clap(long)]
    pub tuner: Option<TunerSpec>,

    pub command: PathBuf,
    pub args: Vec<String>,
}

impl RunOpt {
    pub fn run(&self) -> anyhow::Result<()> {
        let command = CommandSpec {
            path: self.command.clone(),
            args: self.args.clone(),
        };
        let study = StudySpec {
            name: self
                .study_name
                .clone()
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            id: uuid::Uuid::new_v4(),
            attrs: self
                .study_attrs
                .iter()
                .cloned()
                .map(|a| (a.key, a.value))
                .collect(),
            tuner: self.tuner.clone().unwrap_or_default(),
            command,
        };
        let opt = StudyRunnerOpt {
            study,
            workers: self.workers,
            repeat: self.repeat,
        };

        let stdout = std::io::stdout();
        let runner = StudyRunner::new(stdout.lock(), opt)?;
        self.load_then_run(runner)
    }

    fn load_then_run<W: Write>(&self, mut runner: StudyRunner<W>) -> anyhow::Result<()> {
        for path in &self.load {
            self.load(&mut runner, path)
                .with_context(|| format!("Cannot load a study: path={:?}", path))?;
        }
        runner.run()
    }

    fn load<W: Write>(&self, runner: &mut StudyRunner<W>, path: &PathBuf) -> anyhow::Result<()> {
        let file = std::fs::File::open(path)?;
        runner.load_study(EventReader::new(BufReader::new(file)))?;
        Ok(())
    }
}
