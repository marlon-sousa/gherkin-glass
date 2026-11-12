//! Entity: what an output folder's marker records, every file ghg wrote there.

use std::collections::{BTreeMap, BTreeSet};

use crate::entities::VersionName;

/// A file's path below the folder it is in, folders separated by `/`.
pub(crate) type FilePath = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Record {
    /// A site without versions, and its files.
    Site(BTreeSet<FilePath>),
    /// Versions of a site: the files of the versions page, at the output
    /// folder's root, and each version with the files of its folder.
    Versions {
        page: BTreeSet<FilePath>,
        versions: BTreeMap<VersionName, BTreeSet<FilePath>>,
    },
}

impl Record {
    /// The marker of a site without versions, at the output folder's root.
    pub(crate) const SITE_MARKER: &str = "ghg-site.json";
    /// The marker of versions of a site, at the output folder's root.
    pub(crate) const VERSIONS_MARKER: &str = "ghg-versions.json";
}
