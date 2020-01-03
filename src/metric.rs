use crate::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub direction: Direction,
    pub value: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Direction {
    Minimize,
    Maximize,
}

impl Default for Direction {
    fn default() -> Self {
        Self::Minimize
    }
}

impl FromStr for Direction {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "MINIMIZE" => Ok(Self::Minimize),
            "MAXIMIZE" => Ok(Self::Maximize),
            _ => track_panic!(ErrorKind::InvalidInput, "Unknown direction: {:?}", s),
        }
    }
}
