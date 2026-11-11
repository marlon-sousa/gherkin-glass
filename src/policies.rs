//! Policy: the pure decisions of reading the input, of how a build is run, of the anchors, of the pages, the folder pages and the folder tree, of which changes ghg serve rebuilds for, and of highlighting and themes.

mod anchor_rule;
mod argument_parsing;
mod breadcrumb_trail;
mod config_parsing;
mod contrast;
mod doc_string_correction;
mod file_language;
mod file_selection;
mod folder_pages;
mod folder_tree;
mod interface_text;
mod link_address;
mod page_address;
mod page_building;
mod page_frame;
mod page_language;
mod page_rendering;
mod rebuild_choice;
mod settings_resolution;
mod site_title;
mod step_highlighting;
mod step_keyword_correction;
mod tag_linking;
mod tag_naming;
mod tag_pages;
mod theme_stylesheet;
mod themes;
mod unnamed_blocks;

pub use anchor_rule::make_anchors;
pub(crate) use anchor_rule::{anchor_of, unique};
pub(crate) use argument_parsing::parse_arguments;
pub(crate) use breadcrumb_trail::breadcrumb;
pub(crate) use config_parsing::parse_config;
#[cfg(test)]
pub(crate) use contrast::PART_CONTRAST;
pub(crate) use contrast::{TEXT_CONTRAST, contrast_ratio, hundredths};
pub(crate) use doc_string_correction::correct_doc_strings;
pub(crate) use file_language::file_language;
pub(crate) use file_selection::{FileSelection, IGNORE_FILE, parse_rules};
pub(crate) use folder_pages::build_folder_pages;
pub(crate) use folder_tree::{build_folder_tree, render_folder_tree};
pub(crate) use interface_text::CATALOGUE;
pub(crate) use link_address::percent_encoded;
pub(crate) use page_address::{FOLDER_PAGE, page_address, root_from};
pub(crate) use page_building::build_page;
pub(crate) use page_frame::page_frame;
pub(crate) use page_language::page_language;
pub(crate) use page_rendering::{
    render_folder_page, render_page, render_tag_index, render_tag_page,
};
pub(crate) use rebuild_choice::{Read, calls_for_rebuild, remembered};
pub(crate) use settings_resolution::{config_folder, resolve_settings};
pub(crate) use site_title::site_title;
pub use step_highlighting::highlight_step;
pub(crate) use step_keyword_correction::correct_step_keywords;
pub(crate) use tag_linking::{PatternProblem, outside_link, unknown_group, whole_tag_pattern};
pub(crate) use tag_naming::{name_tags, tag_index_address, tags_of};
pub(crate) use tag_pages::{build_tag_index, build_tag_page, tag_pages_language};
pub(crate) use theme_stylesheet::theme_stylesheet;
pub use themes::DEFAULT_ACCENTS;
pub(crate) use themes::{check_accent, palette};
pub(crate) use unnamed_blocks::unnamed_blocks;
