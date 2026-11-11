//! Entity: what stops a build writing its output, said of the output folder or of a version's folder.

use std::path::PathBuf;

use thiserror::Error;

use crate::entities::VersionName;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{}{}: {kind}", output.display(), version.as_ref().map_or(String::new(), |name| format!("/{name}")))]
pub(crate) struct OutputProblem {
    /// The output folder, as the settings give it.
    pub(crate) output: PathBuf,
    /// The version whose folder it is; `None` for the output folder itself.
    pub(crate) version: Option<VersionName>,
    pub(crate) kind: OutputProblemKind,
}

/// What is wrong. The words a person reads come from the interface text
/// catalogue; the `#[error]` text is for developers.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum OutputProblemKind {
    /// The site, or the version, is already built there.
    #[error("already built, and is not to be replaced")]
    AlreadyBuilt,
    /// The output folder of a build without a version holds versions.
    #[error("holds versions")]
    HoldsVersions,
    /// The output folder of a versioned build holds a site without versions.
    #[error("holds a site without versions")]
    HoldsSite,
    /// The output folder of a build without a version holds something
    /// else, or is a file or a link.
    #[error("not a site ghg built")]
    NotASite,
    /// A file, a symbolic link, or a folder with something in it and no marker.
    #[error("not a version ghg built")]
    NotAVersion,
    /// A folder whose name differs from the version's only in its capitals.
    #[error("another name in other capitals")]
    OtherCapitals,
    /// `reason` is the operating system's.
    #[error("cannot be read: {reason}")]
    CannotBeRead { reason: String },
    /// `reason` is the operating system's.
    #[error("cannot be written: {reason}")]
    CannotBeWritten { reason: String },
    /// The output folder's; `reason` is the operating system's.
    #[error("cannot be listed: {reason}")]
    CannotBeListed { reason: String },
}
