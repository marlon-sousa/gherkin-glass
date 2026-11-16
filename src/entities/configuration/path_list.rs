//! Entity: where `--only` reads its list of feature files from.

use std::path::{Path, PathBuf};

/// Where `--only` reads its list from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PathList {
    File(PathBuf),
    /// Given as `-`.
    StandardInput,
}

impl PathList {
    /// What `--only` was given, `-` for standard input.
    pub(crate) fn as_given(&self) -> PathBuf {
        match self {
            PathList::File(file) => file.clone(),
            PathList::StandardInput => PathBuf::from("-"),
        }
    }

    /// The file the list is read from; `None` for standard input.
    pub(crate) fn file(&self) -> Option<&Path> {
        match self {
            PathList::File(file) => Some(file),
            PathList::StandardInput => None,
        }
    }
}
