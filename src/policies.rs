//! Policy: the pure decisions of reading the input.

mod doc_string_reading;
mod file_selection;

pub(crate) use doc_string_reading::read_doc_string;
pub(crate) use file_selection::{FileSelection, IGNORE_FILE, parse_rules};
