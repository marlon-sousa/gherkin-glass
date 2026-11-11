//! Adapter: the output folder's marker, a JSON file of what ghg wrote there.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::entities::{FilePath, Record, VersionName};

/// The marker of a site without versions.
#[derive(Serialize, Deserialize)]
struct SiteMarker {
    files: BTreeSet<FilePath>,
}

/// The marker of versions of a site.
#[derive(Serialize, Deserialize)]
struct VersionsMarker {
    page: BTreeSet<FilePath>,
    versions: BTreeMap<String, BTreeSet<FilePath>>,
}

/// The marker's file name and its text, for `record`.
pub(super) fn marker_text(record: &Record) -> serde_json::Result<(&'static str, String)> {
    match record {
        Record::Site(files) => {
            let marker = SiteMarker {
                files: files.clone(),
            };
            Ok((Record::SITE_MARKER, serde_json::to_string_pretty(&marker)?))
        }
        Record::Versions { page, versions } => {
            let marker = VersionsMarker {
                page: page.clone(),
                versions: versions.iter().map(written_version).collect(),
            };
            Ok((
                Record::VERSIONS_MARKER,
                serde_json::to_string_pretty(&marker)?,
            ))
        }
    }
}

/// What a site's marker, `text`, records; `None` when it is not one.
pub(super) fn site_record(text: &str) -> Option<Record> {
    let marker: SiteMarker = serde_json::from_str(text).ok()?;
    Some(Record::Site(marker.files))
}

/// What a versions marker, `text`, records; `None` when it is not one, or
/// names a version that cannot be one.
pub(super) fn versions_record(text: &str) -> Option<Record> {
    let marker: VersionsMarker = serde_json::from_str(text).ok()?;
    let versions = marker
        .versions
        .into_iter()
        .map(read_version)
        .collect::<Option<_>>()?;
    Some(Record::Versions {
        page: marker.page,
        versions,
    })
}

fn written_version(
    (name, files): (&VersionName, &BTreeSet<FilePath>),
) -> (String, BTreeSet<FilePath>) {
    (name.to_string(), files.clone())
}

/// A version as its marker names it, `None` when its name cannot be a version's.
fn read_version(
    (name, files): (String, BTreeSet<FilePath>),
) -> Option<(VersionName, BTreeSet<FilePath>)> {
    Some((VersionName::new(&name)?, files))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    fn set(paths: &[&str]) -> BTreeSet<FilePath> {
        paths.iter().map(|path| path.to_string()).collect()
    }

    fn versions() -> Record {
        let main = VersionName::new("main").expect("a version name");
        Record::Versions {
            page: set(&["index.html", "assets/a.css"]),
            versions: BTreeMap::from([(main, set(&["index.html", "sub/b.html"]))]),
        }
    }

    #[rstest]
    #[case::a_site(Record::Site(set(&["index.html", "sub/a.html"])), Record::SITE_MARKER)]
    #[case::versions(versions(), Record::VERSIONS_MARKER)]
    fn a_marker_reads_back_as_it_was_written(#[case] record: Record, #[case] file: &str) {
        let (name, text) = marker_text(&record).expect("written");
        assert_eq!(name, file);
        let read = match record {
            Record::Site(_) => site_record(&text),
            Record::Versions { .. } => versions_record(&text),
        };
        assert_eq!(read, Some(record));
    }

    #[rstest]
    #[case::not_json("not a marker")]
    #[case::an_earlier_marker("{\"site\": true}")]
    #[case::a_versions_marker("{\"page\": [], \"versions\": {}}")]
    fn a_site_marker_that_is_not_one_reads_as_none(#[case] text: &str) {
        assert_eq!(site_record(text), None);
    }

    #[rstest]
    #[case::not_json("not a marker")]
    #[case::a_site_marker("{\"files\": []}")]
    #[case::a_name_that_is_no_version("{\"page\": [], \"versions\": {\"a/b\": []}}")]
    fn a_versions_marker_that_is_not_one_reads_as_none(#[case] text: &str) {
        assert_eq!(versions_record(text), None);
    }
}
