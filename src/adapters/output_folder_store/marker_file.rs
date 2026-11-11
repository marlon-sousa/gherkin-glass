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
                versions: versions
                    .iter()
                    .map(|(name, files)| (name.to_string(), files.clone()))
                    .collect(),
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
        .map(|(name, files)| Some((VersionName::new(&name)?, files)))
        .collect::<Option<_>>()?;
    Some(Record::Versions {
        page: marker.page,
        versions,
    })
}
