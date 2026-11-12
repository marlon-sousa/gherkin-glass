//! Output folders as ghg finds them, for the tests of the site writer and the controller.

use std::collections::{BTreeMap, BTreeSet};

use crate::entities::{FilePath, OutputContents, Record, VersionName};

pub(crate) fn name(text: &str) -> VersionName {
    VersionName::new(text).expect("a version name")
}

pub(crate) fn set(paths: &[&str]) -> BTreeSet<FilePath> {
    paths.iter().map(|path| path.to_string()).collect()
}

/// A folder holding `files` and `folders`, whose marker records `record`.
pub(crate) fn folder(files: &[&str], folders: &[&str], record: Option<Record>) -> OutputContents {
    OutputContents::Folder {
        files: set(files),
        folders: set(folders),
        record,
    }
}

/// The output folder of the versions `main` and `v1.0.0`, as ghg wrote it.
pub(crate) fn two_versions() -> OutputContents {
    let versions = BTreeMap::from([
        (name("main"), set(&["index.html"])),
        (name("v1.0.0"), set(&["old.html"])),
    ]);
    let record = Record::Versions {
        page: set(&["index.html"]),
        versions,
    };
    folder(
        &[
            "ghg-versions.json",
            "index.html",
            "main/index.html",
            "v1.0.0/old.html",
        ],
        &["main", "v1.0.0"],
        Some(record),
    )
}
