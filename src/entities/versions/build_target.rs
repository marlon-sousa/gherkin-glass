//! Entity: what a build writes into its output folder, a site without versions or one version of a site.

use crate::entities::VersionName;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BuildTarget {
    /// A site without versions, at the output folder's root.
    Site,
    /// A version, in its folder inside the output folder.
    Version(VersionName),
}
