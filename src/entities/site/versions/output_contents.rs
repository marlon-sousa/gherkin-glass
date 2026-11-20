//! Entity: what is at the place of an output folder, every file and folder in it, and what its marker records.

use std::collections::{BTreeMap, BTreeSet};

use crate::entities::{EntryKind, FilePath, Record};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OutputContents {
    Missing,
    /// A file or a symbolic link, which is never followed.
    NotAFolder,
    Folder {
        /// Every file and symbolic link below the folder, the marker included, but what is kept.
        files: BTreeSet<FilePath>,
        /// Every folder below it, but what is kept.
        folders: BTreeSet<FilePath>,
        /// What the build leaves alone there, with its kind; a folder kept is not looked into.
        kept: BTreeMap<FilePath, EntryKind>,
        /// What its marker records; `None` when it holds no marker that can be read.
        record: Option<Record>,
    },
}
