#[macro_use]
extern crate trackable;

pub use self::error::{Error, ErrorKind};

pub mod obs;
pub mod param;
pub mod samplers;

mod error;

pub type Result<T, E = Error> = std::result::Result<T, E>;
