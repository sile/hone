use crate::envvar;
use crate::param::{
    CategoricalParamType, ContinousParamType, DiscreteParamType, FidelityParamType,
    NormalParamType, NumParamType, OrdinalParamType, ParamName, ParamType, StrParamType,
};
use crate::rpc;
use anyhow::Context;

#[derive(Debug, clap::Args)]
pub struct AskOpt {
    pub param_name: String,

    #[clap(long, short = 'l')]
    pub long_option: bool,

    #[clap(subcommand)]
    pub param_spec: ParamSpec,
}

impl AskOpt {
    pub fn ask(&self) -> anyhow::Result<String> {
        let observation_id = envvar::get_observation_id()?;
        let param_type = self
            .param_spec
            .to_param_type()
            .with_context(|| format!("the specification of {:?} is invalid", self.param_name))?;
        let req = rpc::AskReq {
            observation_id,
            param_name: ParamName::new(self.param_name.clone()),
            param_type,
        };
        let res = rpc::call::<rpc::AskRpc>(req)?;
        let v = res.to_string();
        if self.long_option {
            if matches!(self.param_spec, ParamSpec::Bool) && v == "true" {
                Ok(format!("--{}", self.param_name))
            } else {
                Ok(format!("--{}={:?}", self.param_name, v))
            }
        } else {
            Ok(v)
        }
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum ParamSpec {
    Bool,
    Choice {
        choices: Vec<String>,
        #[clap(long)]
        ordinal: bool,
    },
    Range {
        min: f64,
        max: f64,
        #[clap(long)]
        ln: bool,
        #[clap(long)]
        step: Option<f64>,
        #[clap(long)]
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
            Self::Bool => CategoricalParamType::new(vec!["false".to_owned(), "true".to_owned()])
                .map(StrParamType::Categorical)
                .map(ParamType::Str),
            Self::Choice {
                choices,
                ordinal: false,
            } => CategoricalParamType::new(choices.clone())
                .map(StrParamType::Categorical)
                .map(ParamType::Str),
            Self::Choice {
                choices,
                ordinal: true,
            } => OrdinalParamType::new(choices.clone())
                .map(StrParamType::Ordinal)
                .map(ParamType::Str),
            Self::Normal { mean, stddev } => NormalParamType::new(*mean, *stddev)
                .map(NumParamType::Normal)
                .map(ParamType::Num),
            Self::Range {
                min,
                max,
                ln,
                step: None,
                fidelity: false,
            } => ContinousParamType::new(*min, *max, *ln)
                .map(NumParamType::Continous)
                .map(ParamType::Num),
            Self::Range {
                min,
                max,
                ln: false,
                step: Some(step),
                fidelity: false,
            } => DiscreteParamType::new(*min, *max, *step)
                .map(NumParamType::Discrete)
                .map(ParamType::Num),
            Self::Range {
                min,
                max,
                ln: false,
                step,
                fidelity: true,
            } => FidelityParamType::new(*min, *max, *step)
                .map(NumParamType::Fidelity)
                .map(ParamType::Num),
            Self::Range {
                ln: true,
                step: Some(_),
                ..
            } => anyhow::bail!("Cannot specify both `--ln` and `--step` options."),
            Self::Range {
                ln: true,
                fidelity: true,
                ..
            } => anyhow::bail!("Cannot specify both `--ln` and `--fidelity` options."),
        }
    }
}
