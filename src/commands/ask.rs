use crate::domain::ParamType;
use crate::envvar;

#[derive(Debug, structopt::StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct AskOpt {
    pub param_name: String,

    #[structopt(long, short = "l")]
    pub long_option: bool,

    #[structopt(subcommand)]
    pub param_spec: ParamSpec,
}

impl AskOpt {
    pub fn ask(&self) -> anyhow::Result<String> {
        let run_id = envvar::get_run_id()?;
        let param_type = self.param_spec.to_param_type()?;
        todo!()
    }
}

#[derive(Debug, structopt::StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum ParamSpec {
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

impl ParamSpec {
    fn to_param_type(&self) -> anyhow::Result<ParamType> {
        match self {
            Self::Bool => Ok(ParamType::categorical(vec![
                "false".to_owned(),
                "true".to_owned(),
            ])),
            Self::Choice {
                choices,
                ordinal: false,
            } => Ok(ParamType::categorical(choices.clone())),
            Self::Choice {
                choices,
                ordinal: true,
            } => Ok(ParamType::ordinal(choices.clone())),
            Self::Normal { mean, stddev } => Ok(ParamType::normal(*mean, *stddev)),
            Self::Range {
                start,
                end,
                ln,
                step: None,
                fidelity,
            } => Ok(ParamType::continous(*start, *end, *ln, *fidelity)),
            Self::Range {
                start,
                end,
                ln: false,
                step: Some(step),
                fidelity,
            } => Ok(ParamType::discrete(*start, *end, *step, *fidelity)),
            Self::Range {
                ln: true,
                step: Some(_),
                ..
            } => anyhow::bail!("Cannot specify both `--ln` and `--step` options."),
        }
    }
}
