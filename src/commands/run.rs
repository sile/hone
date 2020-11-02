use crate::runner::{CommandRunnerOpt, StudyRunner, StudyRunnerOpt};
use std::num::NonZeroUsize;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct RunOpt {
    // TODO: Implement
    #[structopt(long)]
    pub tempdir: bool,

    #[structopt(long, default_value = "1")]
    pub workers: NonZeroUsize,

    #[structopt(long)]
    pub runs: Option<usize>,

    // TODO: seed
    // TODO: timeout, search-space, retry, sync
    pub command: String,
    pub args: Vec<String>,
}

impl RunOpt {
    pub fn run(self) -> anyhow::Result<()> {
        let opt = StudyRunnerOpt {
            study_name: uuid::Uuid::new_v4().to_string(),
            workers: self.workers,
            runs: self.runs,
            command: CommandRunnerOpt {
                path: self.command.clone(),
                args: self.args.clone(),
            },
        };
        let stdout = std::io::stdout();
        let runner = StudyRunner::new(stdout.lock(), opt)?;
        runner.run()
    }
}
