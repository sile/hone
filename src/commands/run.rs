use crate::attr::Attr;
use crate::optimizer::OptimizerSpec;
use crate::runner::{CommandRunnerOpt, StudyRunner, StudyRunnerOpt};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct RunOpt {
    #[structopt(long, default_value = "1")]
    pub workers: NonZeroUsize,

    #[structopt(long, short = "n")]
    pub repeat: Option<usize>,

    #[structopt(long, parse(try_from_str = crate::json::parse_json))]
    pub optim: Option<OptimizerSpec>,

    #[structopt(long = "name")]
    pub study_name: Option<String>,

    #[structopt(long, short = "o")]
    pub output: Option<PathBuf>,

    #[structopt(long)]
    pub attrs: Vec<Attr>,

    // TODO: support multiple paths
    #[structopt(long)]
    pub resume: Option<PathBuf>,

    pub command: String,
    pub args: Vec<String>,
}

impl RunOpt {
    pub fn run(self) -> anyhow::Result<()> {
        let opt = StudyRunnerOpt {
            study_name: self
                .study_name
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            study_instance: uuid::Uuid::new_v4(),
            resume: self.resume,
            attrs: self.attrs.into_iter().map(|a| (a.key, a.value)).collect(),
            workers: self.workers,
            runs: self.repeat,
            output: self.output.clone(),
            command: CommandRunnerOpt {
                path: self.command.clone(),
                args: self.args.clone(),
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
