//! Entity: the collecting stage, the input: the entries of its folders, what collecting it found, each feature as the parser returned it and as it was read.

mod collected_input;
mod doc_string;
mod folder_entry;
mod ignore_file;
mod page_clash;
mod parsed_feature;
mod raw_feature;

pub(crate) use collected_input::CollectedInput;
pub(crate) use doc_string::DocString;
pub(crate) use folder_entry::{EntryKind, FolderEntry};
pub(crate) use ignore_file::IGNORE_FILE;
pub(crate) use page_clash::PageClash;
pub(crate) use parsed_feature::ParsedFeature;
pub(crate) use raw_feature::RawFeature;
