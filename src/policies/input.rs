//! Policy: reading the input, which files are read, in what language, what is corrected in what the parser gives, and what stops a build.

mod doc_string_correction;
mod file_language;
mod file_selection;
mod page_collisions;
mod step_keyword_correction;
mod unnamed_blocks;

pub(crate) use doc_string_correction::correct_doc_strings;
pub(crate) use file_language::file_language;
pub(crate) use file_selection::{FileSelection, IGNORE_FILE, InvalidRule, parse_rules};
pub(crate) use page_collisions::page_collisions;
pub(crate) use step_keyword_correction::correct_step_keywords;
pub(crate) use unnamed_blocks::unnamed_blocks;
