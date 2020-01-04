#[macro_use]
extern crate trackable;

pub use self::error::{Error, ErrorKind};

pub mod config;
pub mod metric;
pub mod optimizer;
pub mod param;
pub mod pubsub;
pub mod study;
pub mod trial;

// commands
pub mod get;
pub mod init;
pub mod report;
pub mod run;
pub mod studies;

mod error;

pub type Result<T, E = Error> = std::result::Result<T, E>;
