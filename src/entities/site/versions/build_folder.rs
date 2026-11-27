//! Entity: an output folder that nothing stops a build from writing into, what the build writes there, and what its store holds.

use std::path::{Path, PathBuf};

use crate::entities::{BuildTarget, PublishedStore, Record};

/// Outside the crate, made only by [`crate::services::SiteWriter`]'s checks, so
/// that no other folder is ever emptied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BuildFolder {
    output: PathBuf,
    target: BuildTarget,
    record: Option<Record>,
    published: PublishedStore,
}

impl BuildFolder {
    /// The output folder `output`, found safe to write `target` into, what
    /// its marker recorded then, `None` when it was missing or empty, and
    /// what its store held then.
    pub(crate) fn new(
        output: &Path,
        target: BuildTarget,
        record: Option<Record>,
        published: PublishedStore,
    ) -> Self {
        BuildFolder {
            output: output.to_path_buf(),
            target,
            record,
            published,
        }
    }

    pub(crate) fn output(&self) -> &Path {
        &self.output
    }

    pub(crate) fn target(&self) -> &BuildTarget {
        &self.target
    }

    pub(crate) fn record(&self) -> Option<&Record> {
        self.record.as_ref()
    }

    pub(crate) fn published(&self) -> &PublishedStore {
        &self.published
    }
}
