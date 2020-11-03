use crate::optimizer::OptimizerSpec;
use crate::runner::{CommandRunnerOpt, StudyRunner, StudyRunnerOpt};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct RunOpt {
    // TODO: Implement
    #[structopt(long)]
    pub tempdir: bool,

    #[structopt(long, default_value = "1")]
    pub workers: NonZeroUsize,

    #[structopt(long, short = "n")]
    pub repeat: Option<usize>,

    #[structopt(long, parse(try_from_str = crate::json::parse_json))]
    pub optim: Option<OptimizerSpec>,

    #[structopt(long = "name")]
    pub study_name: Option<String>,

    #[structopt(long)]
    pub nocapture_stdout: bool,

    #[structopt(long)]
    pub nocapture_stderr: bool,

    #[structopt(long, short = "o")]
    pub output: Option<PathBuf>,

    // TODO: attrs
    // TODO: inherit | resume
    // TODO: timeout
    pub command: String,
    pub args: Vec<String>,
}

impl RunOpt {
    pub fn run(self) -> anyhow::Result<()> {
        let opt = StudyRunnerOpt {
            study_name: self
                .study_name
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            workers: self.workers,
            runs: self.repeat,
            output: self.output.clone(),
            command: CommandRunnerOpt {
                path: self.command.clone(),
                args: self.args.clone(),
                nocapture_stdout: self.nocapture_stdout,
                nocapture_stderr: self.nocapture_stderr,
            },
        };
        let optimizer = self.optim.clone().unwrap_or_default().build()?;

        if let Some(path) = self.output {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)?;
            }
            let file = std::fs::File::create(path)?;
            let runner = StudyRunner::new(file, optimizer, opt)?;
            runner.run()
        } else {
            let stdout = std::io::stdout();
            let runner = StudyRunner::new(stdout.lock(), optimizer, opt)?;
            runner.run()
        }
    }
}
