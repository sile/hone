use crate::envvar;
use crate::rpc;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

pub type InternalHpValue = f64;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct HpOpt {
    #[structopt(flatten)]
    pub spec: HpSpec,

    #[structopt(long)]
    pub arg: bool, // eval
                   // TODO: server_addr, trial_id
}

impl HpOpt {
    pub fn ask(&self) -> anyhow::Result<String> {
        let trial_id = std::env::var(envvar::KEY_TRIAL_ID)?.parse()?;
        let req = rpc::AskReq {
            trial_id,
            param_name: self.spec.name.clone(),
            distribution: self.spec.distribution.clone(),
        };
        let res = rpc::call::<rpc::AskRpc>(req)??;
        let value = if self.arg {
            match res {
                HpValue::Flag(false) => format!(""),
                HpValue::Flag(true) => format!("--{}", self.spec.name),
                HpValue::Choice(v) => format!("--{}={:?}", self.spec.name, v),
                HpValue::Range(v) => format!("--{}={}", self.spec.name, v),
                HpValue::Normal(v) => format!("--{}={}", self.spec.name, v),
            }
        } else {
            match res {
                HpValue::Flag(v) => v.to_string(),
                HpValue::Choice(v) => v,
                HpValue::Range(v) => v.to_string(),
                HpValue::Normal(v) => v.to_string(),
            }
        };
        Ok(value)
    }
}

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub struct HpSpec {
    pub name: String,

    #[structopt(subcommand)]
    pub distribution: Option<HpDistribution>,
}

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub enum HpDistribution {
    Flag,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HpValue {
    Flag(bool),
    Choice(String),
    Range(f64),
    Normal(f64),
}
