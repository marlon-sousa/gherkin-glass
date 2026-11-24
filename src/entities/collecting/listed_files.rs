//! Entity: what the list of `--only` named, which an input tree is walked for: no path at all, or the files it names below the input folder.

use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ListedFiles {
    /// The list named no path, being empty or of blank lines only.
    NoPath,
    /// The paths it named below the input folder, none when every path it named is outside it.
    Below(BTreeSet<PathBuf>),
}

impl ListedFiles {
    /// The paths named below the input folder, none for a list that named no path.
    pub(crate) fn files(&self) -> BTreeSet<PathBuf> {
        match self {
            ListedFiles::NoPath => BTreeSet::new(),
            ListedFiles::Below(files) => files.clone(),
        }
    }
}
