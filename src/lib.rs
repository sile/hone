#[macro_use]
extern crate trackable;

pub use self::error::{Error, ErrorKind};

pub mod config;
pub mod param;
pub mod study;

// commands
pub mod get;
pub mod init;
pub mod run;

// pub mod var;
// pub mod history;
// pub mod obs;
// pub mod observe;
// pub mod samplers;

mod error;

pub type Result<T, E = Error> = std::result::Result<T, E>;
