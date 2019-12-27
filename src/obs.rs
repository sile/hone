use crate::param::{Param, ParamValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Obs {
    pub param: ParamValue,
    // TODO: intermediate values
    // TODO: timestamps
    pub values: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Record {
    Start, // TODO: timestamp
    Param { spec: Param, value: ParamValue },
    Value { values: Vec<f64> },
    End, // TODO: timestamp, state
}
