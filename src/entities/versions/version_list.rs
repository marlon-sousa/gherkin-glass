//! Entity: the versions in an output folder, in the order the versions page lists them.

use crate::entities::VersionName;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct VersionList {
    /// The names that are not version numbers, sorted by name.
    pub(crate) development: Vec<VersionName>,
    /// Newest first.
    pub(crate) releases: Vec<VersionName>,
    /// Newest first.
    pub(crate) pre_releases: Vec<VersionName>,
}
