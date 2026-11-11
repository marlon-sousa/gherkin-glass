//! Entity: how a build is run, what the command line asks for, what a configuration file sets, and the settings resolved from them.

mod config;
mod invocation;
mod settings;
mod unnamed;

pub(crate) use config::Config;
pub(crate) use invocation::{BuildOptions, Invocation, ServeMode, ServeOptions};
pub(crate) use settings::Settings;
pub(crate) use unnamed::Unnamed;
