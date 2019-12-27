use crate::obs::Obs;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History(Vec<Obs>);

impl History {
    // TODO: optimize
    pub fn load<P: AsRef<Path>>(dir: P) -> Result<Self> {
        panic!()
    }

    pub fn save<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
        panic!()
    }
}
