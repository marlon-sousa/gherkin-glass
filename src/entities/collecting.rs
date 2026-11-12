//! Entity: the collecting stage, the input: the entries of its folders, the input tree the walk makes of them, the rules of their `.ghg-ignore` files and the revisions of their files, what the paths that changed make of the last tree, each feature as the parser returned it and as it was read.

mod collected_changes;
mod doc_string;
mod folder_entry;
mod ignore_file;
mod input_tree;
mod page_clash;
mod parsed_feature;
mod raw_feature;
mod revision;

pub(crate) use collected_changes::CollectedChanges;

pub(crate) use doc_string::DocString;
pub(crate) use folder_entry::{EntryKind, FolderEntry};
pub(crate) use ignore_file::{IGNORE_FILE, IgnoreRules};
pub(crate) use input_tree::{
    FileToParse, InputEntry, InputTree, InputTreeBuilder, ListedFolder, ParsedFile, RevisedFeature,
};
pub(crate) use page_clash::PageClash;
pub(crate) use parsed_feature::ParsedFeature;
pub(crate) use raw_feature::RawFeature;
pub(crate) use revision::Revision;
