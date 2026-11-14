//! Entity: what an output folder's marker records, every file ghg wrote there.

use std::collections::{BTreeMap, BTreeSet};

use crate::entities::{VersionListing, VersionName};

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
        /// The versions built as previews, which the versions page does not list.
        previews: BTreeSet<VersionName>,
    },
}

impl Record {
    /// The marker of a site without versions, at the output folder's root.
    pub(crate) const SITE_MARKER: &str = "ghg-site.json";
    /// The marker of versions of a site, at the output folder's root.
    pub(crate) const VERSIONS_MARKER: &str = "ghg-versions.json";

    /// The versions the versions page lists: every one recorded but the
    /// previews, none for a site without versions.
    pub(crate) fn listed_versions(&self) -> BTreeSet<VersionName> {
        match self {
            Record::Versions {
                versions, previews, ..
            } => versions
                .keys()
                .cloned()
                .collect::<BTreeSet<_>>()
                .difference(previews)
                .cloned()
                .collect(),
            Record::Site(_) => BTreeSet::new(),
        }
    }

    /// The versions of `last`, if any, once the version `name`, with `files`,
    /// is written beside the versions page of `page`, among the previews or
    /// not as `listing` says.
    pub(crate) fn with_version(
        last: Option<&Record>,
        page: BTreeSet<FilePath>,
        (name, files): (&VersionName, BTreeSet<FilePath>),
        listing: VersionListing,
    ) -> Record {
        let (mut versions, mut previews) = match last {
            Some(Record::Versions {
                versions, previews, ..
            }) => (versions.clone(), previews.clone()),
            Some(Record::Site(_)) | None => (BTreeMap::new(), BTreeSet::new()),
        };
        versions.insert(name.clone(), files);
        match listing {
            VersionListing::Listed => previews.remove(name),
            VersionListing::Preview => previews.insert(name.clone()),
        };
        Record::Versions {
            page,
            versions,
            previews,
        }
    }
}
