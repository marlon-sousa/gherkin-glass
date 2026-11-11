//! Policy: the pure decisions of reading the input, of how a build is run, and of the anchors.

mod anchor_rule;
mod argument_parsing;
mod config_parsing;
mod doc_string_correction;
mod file_selection;
mod interface_text;
mod settings_resolution;
mod unnamed_blocks;

pub use anchor_rule::make_anchors;
pub(crate) use argument_parsing::parse_arguments;
pub(crate) use config_parsing::parse_config;
pub(crate) use doc_string_correction::correct_doc_strings;
pub(crate) use file_selection::{FileSelection, IGNORE_FILE, parse_rules};
pub(crate) use interface_text::CATALOGUE;
pub(crate) use settings_resolution::{config_folder, resolve_settings};
pub(crate) use unnamed_blocks::unnamed_blocks;
