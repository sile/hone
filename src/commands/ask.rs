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
        let obs_id = envvar::get_observation_id()?;
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
}
