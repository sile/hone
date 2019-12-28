#[macro_use]
extern crate trackable;

pub use self::error::{Error, ErrorKind};

pub mod config;
pub mod history;
pub mod init;
pub mod obs;
pub mod observe;
pub mod param;
pub mod samplers;

mod error;

pub type Result<T, E = Error> = std::result::Result<T, E>;
