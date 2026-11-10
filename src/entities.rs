//! Entity: the values the build reads and reports.

mod doc_string;
mod folder_entry;
mod parse_failure;
mod parsed_feature;
mod problem;

pub(crate) use doc_string::DocString;
pub use folder_entry::{EntryKind, FolderEntry};
pub use parse_failure::ParseFailure;
pub use parsed_feature::ParsedFeature;
pub use problem::{Problem, Problems};
