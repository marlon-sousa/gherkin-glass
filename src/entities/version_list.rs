//! Entity: the versions in an output folder, in the order the versions page lists them.

use crate::entities::VersionName;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VersionList {
    /// The names that are not version numbers, sorted by name.
    pub development: Vec<VersionName>,
    /// Newest first.
    pub releases: Vec<VersionName>,
    /// Newest first.
    pub pre_releases: Vec<VersionName>,
}

impl VersionList {
    pub fn is_empty(&self) -> bool {
        self.development.is_empty() && self.releases.is_empty() && self.pre_releases.is_empty()
    }
}
