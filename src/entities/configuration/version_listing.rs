//! Entity: whether the version a build makes is listed on the versions page, or is a preview, reached only through its link and not indexed.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum VersionListing {
    #[default]
    Listed,
    /// Given `--preview`.
    Preview,
}
