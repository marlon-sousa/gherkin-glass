//! Entity: what a command writes into its output folder, or removes from it: a site without versions, a version of a site, or a preview.

use crate::entities::VersionName;

/// Sorted as the marker and the store list them: the site, then the versions, then the previews.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum BuildTarget {
    /// A site without versions, its shell at the output folder's root.
    #[default]
    Site,
    /// A version, its shell in its folder at the output folder's root.
    Version(VersionName),
    /// A preview, its shell in its folder inside `previews/`.
    Preview(VersionName),
}

impl BuildTarget {
    /// The version's name, `None` for a site and a preview.
    pub(crate) fn version(&self) -> Option<&VersionName> {
        match self {
            BuildTarget::Version(name) => Some(name),
            BuildTarget::Site | BuildTarget::Preview(_) => None,
        }
    }

    /// The folder at the output folder's root that holds the shell, `None` for a site without
    /// versions, whose shell is at the root.
    pub(crate) fn folder(&self) -> Option<String> {
        match self {
            BuildTarget::Site => None,
            BuildTarget::Version(name) => Some(name.to_string()),
            BuildTarget::Preview(name) => Some(format!("{}/{name}", VersionName::PREVIEWS)),
        }
    }

    /// The preview's name, `None` for a site and a version.
    pub(crate) fn preview(&self) -> Option<&VersionName> {
        match self {
            BuildTarget::Preview(name) => Some(name),
            BuildTarget::Site | BuildTarget::Version(_) => None,
        }
    }
}
