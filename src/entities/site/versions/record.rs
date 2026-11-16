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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    fn name(text: &str) -> VersionName {
        VersionName::new(text).expect("a version name")
    }

    fn names(texts: &[&str]) -> BTreeSet<VersionName> {
        texts.iter().copied().map(name).collect()
    }

    /// The versions `main` and `pr-1`, of no files, with `previews` as previews.
    fn record(previews: &[&str]) -> Record {
        Record::Versions {
            page: BTreeSet::new(),
            versions: BTreeMap::from([
                (name("main"), BTreeSet::new()),
                (name("pr-1"), BTreeSet::new()),
            ]),
            previews: names(previews),
        }
    }

    // Rule 212: the versions page lists every version but the previews.
    #[rstest]
    #[case::no_preview(record(&[]), &["main", "pr-1"])]
    #[case::a_preview(record(&["pr-1"]), &["main"])]
    #[case::every_version_a_preview(record(&["main", "pr-1"]), &[])]
    #[case::a_site_without_versions(Record::Site(BTreeSet::new()), &[])]
    fn the_versions_listed_are_every_one_but_the_previews(
        #[case] record: Record,
        #[case] listed: &[&str],
    ) {
        assert_eq!(record.listed_versions(), names(listed));
    }

    // Rule 212: a version written as a preview is among the previews, and one written without
    // is not, whatever it was before.
    #[rstest]
    #[case::a_new_preview(None, "pr-2", VersionListing::Preview, &["pr-2"])]
    #[case::a_preview_kept(Some(record(&["pr-1"])), "main", VersionListing::Listed, &["pr-1"])]
    #[case::a_preview_listed_again(Some(record(&["pr-1"])), "pr-1", VersionListing::Listed, &[])]
    #[case::a_listed_version_made_a_preview(Some(record(&[])), "main", VersionListing::Preview, &["main"])]
    fn a_version_written_is_recorded_among_the_previews_as_its_listing_says(
        #[case] last: Option<Record>,
        #[case] version: &str,
        #[case] listing: VersionListing,
        #[case] previews: &[&str],
    ) {
        let files = BTreeSet::from(["index.html".to_string()]);
        let written = Record::with_version(
            last.as_ref(),
            BTreeSet::new(),
            (&name(version), files.clone()),
            listing,
        );
        let Record::Versions {
            versions,
            previews: recorded,
            ..
        } = written
        else {
            panic!("{written:?}");
        };
        assert_eq!(versions.get(&name(version)), Some(&files));
        assert_eq!(recorded, names(previews));
    }
}
