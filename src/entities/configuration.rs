//! Entity: how a build is run, what the command line asks for, what a configuration file sets and what is wrong with it, and the settings resolved from them.

mod config;
mod config_problem;
mod invocation;
mod settings;
mod unnamed;

pub(crate) use config::Config;
pub(crate) use config_problem::{ConfigProblem, ConfigProblemKind, ValueType};
pub(crate) use invocation::{ArgumentProblem, BuildOptions, Invocation, ServeMode, ServeOptions};
pub(crate) use settings::Settings;
pub(crate) use unnamed::Unnamed;
