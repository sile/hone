use crate::Result;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SafeString(String);

impl SafeString {
    pub fn new(s: &str) -> Result<Self> {
        panic!()
    }
}

impl Deref for EscapedString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}
