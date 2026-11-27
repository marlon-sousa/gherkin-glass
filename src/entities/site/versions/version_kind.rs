//! Entity: the kind of version a version's name makes.

/// What a version's name makes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VersionKind {
    /// A name that is not a version number, such as `main`.
    Development,
    /// A version number without a pre-release part.
    Release,
    /// A version number with a pre-release part, such as `2.0.0-rc.1`.
    PreRelease,
}

impl VersionKind {
    /// As the descriptor's `version_kind` names it, such as `pre-release`.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            VersionKind::Development => "development",
            VersionKind::Release => "release",
            VersionKind::PreRelease => "pre-release",
        }
    }
}
