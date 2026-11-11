//! Policy: how a build is run, the command line, the cargo plugin's arguments, the configuration file, and the settings resolved from them.

mod argument_parsing;
mod cargo_plugin_arguments;
mod config_parsing;
mod settings_resolution;

pub(crate) use argument_parsing::parse_arguments;
pub(crate) use cargo_plugin_arguments::cargo_plugin_arguments;
pub(crate) use config_parsing::parse_config;
pub(crate) use settings_resolution::{config_folder, resolve_settings};
