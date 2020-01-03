use serde::{Deserialize, Serialize};
use std::fmt;
use structopt::StructOpt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub spec: ParamSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize, StructOpt)]
#[structopt(rename_all = "kebab-case")]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ParamSpec {
    Choice { choices: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamValue(pub String);

impl fmt::Display for ParamValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
