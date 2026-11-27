//! Entity: what is at the place of an output folder, every file and folder in it, and the text of its marker.

use std::collections::{BTreeMap, BTreeSet};

use crate::entities::{EntryKind, FilePath};

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
        /// What the command leaves alone there, with its kind; a folder kept is not looked into.
        kept: BTreeMap<FilePath, EntryKind>,
        /// The text of its marker; `None` when it holds no marker that is a file of UTF-8.
        marker: Option<String>,
    },
}
