//! Entity: how a command that nothing stopped ended: with the site written or with nothing to build, or with a version or a preview removed or with none of that name.

use std::path::PathBuf;

use super::BuiltSite;
use crate::entities::{BuildTarget, NothingToBuild};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BuildReport {
    /// `target` was written into `output`, and `site` is what the build says of it.
    Written {
        site: BuiltSite,
        output: PathBuf,
        target: BuildTarget,
    },
    /// No feature file was read, so nothing was written.
    NothingToBuild(NothingToBuild),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RemoveReport {
    /// `target`, a version or a preview, was removed from `output`.
    Removed {
        output: PathBuf,
        target: BuildTarget,
    },
    /// `output` holds no `target`, or is missing, so nothing was removed.
    NothingToRemove {
        output: PathBuf,
        target: BuildTarget,
    },
}
