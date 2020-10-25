use crate::envvar;

#[derive(Debug, structopt::StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct AskOpt {
    pub param_name: String,

    #[structopt(long, short = "l")]
    pub long_option: bool,

    #[structopt(subcommand)]
    pub param_value: ParamValueSpec,
}

impl AskOpt {
    pub fn ask(&self) -> anyhow::Result<String> {
        let trial_id = envvar::get_trial_id()?;
        todo!()
    }
}

#[derive(Debug, structopt::StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum ParamValueSpec {
    Bool,
    Choice {
        choices: Vec<String>,
        #[structopt(long)]
        ordinal: bool,
    },
    Range {
        start: f64,
        end: f64,
        #[structopt(long)]
        ln: bool,
        #[structopt(long)]
        step: Option<f64>,
        #[structopt(long)]
        fidelity: bool,
    },
    Normal {
        mean: f64,
        stddev: f64,
    },
    RandomSeed {
        #[structopt(long, default_value = "0")]
        min: u64,

        #[structopt(long, default_value = "4294967295")]
        max: u64,
    },
}
