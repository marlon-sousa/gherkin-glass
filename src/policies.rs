//! Policy: the pure decisions of reading the input.

mod doc_string_correction;
mod file_selection;

pub(crate) use doc_string_correction::correct_doc_strings;
pub(crate) use file_selection::{FileSelection, IGNORE_FILE, parse_rules};
