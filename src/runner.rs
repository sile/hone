use std::io::Write;
use std::num::NonZeroUsize;

#[derive(Debug)]
pub struct CommandRunnerOpt {
    pub path: String,
    pub args: Vec<String>,
}

#[derive(Debug)]
pub struct StudyRunnerOpt {
    pub study_name: String,
    pub workers: NonZeroUsize,
    pub runs: usize,
    pub command: CommandRunnerOpt,
}

#[derive(Debug)]
pub struct StudyRunner<W> {
    output: W,
    opt: StudyRunnerOpt,
}

impl<W> StudyRunner<W>
where
    W: Write,
{
    pub fn new(output: W, opt: StudyRunnerOpt) -> Self {
        Self { output, opt }
    }

    pub fn run(self) -> anyhow::Result<()> {
        todo!()
    }
}
