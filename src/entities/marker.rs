//! Entity: what a marker file says a folder is, a site or a version of one, that gherkin-glass built.

use crate::entities::VersionName;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Marker {
    /// An output folder holding a site without versions.
    Site,
    /// A version's folder inside an output folder.
    Version(VersionName),
}
