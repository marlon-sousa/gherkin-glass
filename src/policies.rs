//! Policy: the pure decisions of reading the input, of how a build is run, of the anchors, and of the pages.

mod anchor_rule;
mod argument_parsing;
mod config_parsing;
mod doc_string_correction;
mod file_language;
mod file_selection;
mod interface_text;
mod page_address;
mod page_building;
mod page_language;
mod page_rendering;
mod settings_resolution;
mod step_keyword_correction;
mod unnamed_blocks;

pub use anchor_rule::make_anchors;
pub(crate) use argument_parsing::parse_arguments;
pub(crate) use config_parsing::parse_config;
pub(crate) use doc_string_correction::correct_doc_strings;
pub(crate) use file_language::file_language;
pub(crate) use file_selection::{FileSelection, IGNORE_FILE, parse_rules};
pub(crate) use interface_text::CATALOGUE;
pub(crate) use page_address::{page_address, root_from};
pub(crate) use page_building::build_page;
pub(crate) use page_language::page_language;
pub(crate) use page_rendering::render_page;
pub(crate) use settings_resolution::{config_folder, resolve_settings};
pub(crate) use step_keyword_correction::correct_step_keywords;
pub(crate) use unnamed_blocks::unnamed_blocks;
