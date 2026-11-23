//! Entity: how a build that nothing stopped ended, with the site written or with nothing to build.

use std::path::PathBuf;

use super::BuiltSite;
use crate::entities::{NothingToBuild, VersionName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BuildReport {
    /// The site was written into `output`, as the version `version` when there is one, and
    /// `site` is what the build says of it.
    Written {
        site: BuiltSite,
        output: PathBuf,
        version: Option<VersionName>,
    },
    /// No feature file was read, so nothing was written.
    NothingToBuild(NothingToBuild),
}
