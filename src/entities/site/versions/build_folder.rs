//! Entity: an output folder that nothing stops a command from writing into, what its marker lists, what it holds, and what its store holds.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::entities::{FilePath, PublishedStore};

/// Outside the crate, made only by [`crate::services::SiteWriter`]'s checks, so that nothing is
/// removed from another folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BuildFolder {
    output: PathBuf,
    listed: BTreeSet<FilePath>,
    found: BTreeSet<FilePath>,
    published: PublishedStore,
}

impl BuildFolder {
    /// The output folder `output`, found safe, the files its marker `listed`, none when it was
    /// missing or empty, the files `found` in it, and what its store held then.
    pub(crate) fn new(
        output: &Path,
        listed: BTreeSet<FilePath>,
        found: BTreeSet<FilePath>,
        published: PublishedStore,
    ) -> Self {
        BuildFolder {
            output: output.to_path_buf(),
            listed,
            found,
            published,
        }
    }

    pub(crate) fn output(&self) -> &Path {
        &self.output
    }

    /// Every file the marker lists, some of which may be gone.
    pub(crate) fn listed(&self) -> &BTreeSet<FilePath> {
        &self.listed
    }

    /// Every file in the folder, but what is kept.
    pub(crate) fn found(&self) -> &BTreeSet<FilePath> {
        &self.found
    }

    pub(crate) fn published(&self) -> &PublishedStore {
        &self.published
    }
}
