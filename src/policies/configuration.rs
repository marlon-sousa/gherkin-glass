//! Policy: how a build is run, the command line, the cargo plugin's arguments, the configuration file, the list of feature files `--only` reads, and the settings resolved from them, with what the command line and the file both check: the accent colours' contrast and the patterns of the links from tags.

mod accent_contrast;
mod argument_parsing;
mod cargo_plugin_arguments;
mod config_parsing;
mod path_listing;
mod settings_resolution;
mod tag_patterns;

pub(crate) use argument_parsing::parse_arguments;
pub(crate) use cargo_plugin_arguments::cargo_plugin_arguments;
pub(crate) use config_parsing::parse_config;
pub(crate) use path_listing::listed_paths;
pub(crate) use settings_resolution::{config_folder, resolve_settings};

use accent_contrast::check_accent;
use tag_patterns::{unknown_group, whole_tag_pattern};
