use crate::envvar;
use crate::rpc;
use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
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

impl HpDistribution {
    pub fn expand_if_need(&mut self, d: &Self) -> anyhow::Result<bool> {
        match (self, d) {
            (Self::Flag, Self::Flag) => Ok(false),
            (
                Self::Choice {
                    choices: choices0,
                    ordinal: false,
                },
                Self::Choice {
                    choices: choices1,
                    ordinal: false,
                },
            ) => {
                let choices2: BTreeSet<_> = choices0.iter().chain(choices1.iter()).collect();
                if choices0.len() == choices2.len() {
                    Ok(false)
                } else {
                    let choices3 = choices2.iter().map(|s| s.to_string()).collect();
                    *choices0 = choices3;
                    Ok(true)
                }
            }
            (
                Self::Normal {
                    mean: m0,
                    stddev: s0,
                },
                Self::Normal {
                    mean: m1,
                    stddev: s1,
                },
            ) if m0 == m1 && s0 == s1 => Ok(false),
            (
                Self::Range {
                    start: s0,
                    end: e0,
                    ln: l0,
                    step: st0,
                    fidelity: f0,
                },
                Self::Range {
                    start: s1,
                    end: e1,
                    ln: l1,
                    step: st1,
                    fidelity: f1,
                },
            ) if l0 == l1 && st0 == st1 && f0 == f1 => {
                if *s0 <= *s1 && *e1 <= *e0 {
                    Ok(false)
                } else {
                    *s0 = s0.min(*s1);
                    *e0 = e0.max(*e1);
                    Ok(true)
                }
            }
            _ => bail!("Incompatible distributions"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HpValue {
    Flag(bool),
    Choice(String),
    Range(f64),
    Normal(f64),
}
