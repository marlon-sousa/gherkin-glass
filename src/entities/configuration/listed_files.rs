//! Entity: the feature files `--only` lists, where the list is read from and the paths it holds.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Where `--only` reads its list from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PathList {
    File(PathBuf),
    /// Given as `-`.
    StandardInput,
}

/// The paths a list holds, each as written, relative to the current folder or absolute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ListedFiles {
    pub(crate) list: PathList,
    pub(crate) paths: BTreeSet<PathBuf>,
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
