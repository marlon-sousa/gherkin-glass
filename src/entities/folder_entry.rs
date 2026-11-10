//! Entity: one entry of a folder of the input, as the input folder port lists it.

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderEntry {
    pub path: PathBuf,
    pub kind: EntryKind,
}

/// A symbolic link is `Other`, whatever it points to, so it is never followed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Folder,
    Other,
}
