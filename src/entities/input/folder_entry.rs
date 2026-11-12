//! Entity: an entry of a folder of the input, as listing the folder found it.

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FolderEntry {
    pub(crate) path: PathBuf,
    pub(crate) kind: EntryKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EntryKind {
    File,
    Folder,
    /// Anything else, a symbolic link among them.
    Other,
}
