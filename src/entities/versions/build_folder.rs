//! Entity: an output folder that nothing stops a build from writing into, and what the build writes there.

use std::path::{Path, PathBuf};

use crate::entities::{BuildTarget, Record};

/// Outside the crate, made only by [`crate::services::SiteWriter`]'s checks, so
/// that no other folder is ever emptied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BuildFolder {
    output: PathBuf,
    target: BuildTarget,
    record: Option<Record>,
}

impl BuildFolder {
    /// The output folder `output`, found safe to write `target` into, and
    /// what its marker recorded then, `None` when it was missing or empty.
    pub(crate) fn new(output: &Path, target: BuildTarget, record: Option<Record>) -> Self {
        BuildFolder {
            output: output.to_path_buf(),
            target,
            record,
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
}
