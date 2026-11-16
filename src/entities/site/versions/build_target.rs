//! Entity: what a build writes into its output folder, a site without versions or one version of a site.

use crate::entities::VersionName;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BuildTarget {
    /// A site without versions, at the output folder's root.
    Site,
    /// A version, in its folder inside the output folder.
    Version(VersionName),
}

impl BuildTarget {
    /// The folder at the top of the output folder that a build of this target never looks
    /// into, writes or removes: a versioned output folder's previews.
    pub(crate) fn left_alone(&self) -> Option<&'static str> {
        match self {
            BuildTarget::Site => None,
            BuildTarget::Version(_) => Some(VersionName::PREVIEWS),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_version_leaves_the_previews_alone() {
        let version = VersionName::new("main").map(BuildTarget::Version);
        assert_eq!(
            version.as_ref().and_then(BuildTarget::left_alone),
            Some("previews")
        );
        assert_eq!(BuildTarget::Site.left_alone(), None);
    }
}
