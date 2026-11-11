//! Entity: what is at the place of an output folder, every file and folder in it, and what its marker records.

use std::collections::BTreeSet;

use crate::entities::{FilePath, Record};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OutputContents {
    Missing,
    /// A file or a symbolic link, which is never followed.
    NotAFolder,
    Folder {
        /// Every file and symbolic link below the folder, the marker included.
        files: BTreeSet<FilePath>,
        /// Every folder below it.
        folders: BTreeSet<FilePath>,
        /// What its marker records; `None` when it holds no marker that can be read.
        record: Option<Record>,
    },
}
