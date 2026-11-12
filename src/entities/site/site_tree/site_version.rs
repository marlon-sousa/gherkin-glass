//! Entity: the version of the site a build makes, as its node of the site tree holds it: its name, and the names of the tags' pages, which the next build compares its own with.

use crate::entities::{TagNames, VersionName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SiteVersion {
    /// `None` for a site without versions.
    pub(crate) name: Option<VersionName>,
    pub(crate) tag_names: TagNames,
}
