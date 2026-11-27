//! Output folders as ghg finds them, for the tests of the output's checks, the site writer and the controller.

use std::collections::{BTreeMap, BTreeSet};

use crate::entities::{FilePath, KeptInOutput, KeptPattern, OutputContents, VersionName};
use crate::policies::marker_json;

pub(crate) fn name(text: &str) -> VersionName {
    VersionName::new(text).expect("a version name")
}

/// The patterns `patterns`, in their order.
pub(crate) fn kept_in_output(patterns: &[&str]) -> KeptInOutput {
    KeptInOutput::new(
        patterns
            .iter()
            .map(|text| KeptPattern::new(text).expect("a valid pattern"))
            .collect(),
    )
}

pub(crate) fn set(paths: &[&str]) -> BTreeSet<FilePath> {
    paths.iter().map(|path| path.to_string()).collect()
}

/// A folder holding `files` and `folders`, whose marker's text is `marker`.
pub(crate) fn folder(files: &[&str], folders: &[&str], marker: Option<String>) -> OutputContents {
    OutputContents::Folder {
        files: set(files),
        folders: set(folders),
        kept: BTreeMap::new(),
        marker,
    }
}

/// The text of a marker of the store's format that lists `files`.
pub(crate) fn marker(files: &[&str]) -> String {
    String::from_utf8(marker_json(&set(files))).expect("UTF-8")
}
