//! Policy: the output folder's marker, written as compact JSON from the files ghg wrote there, and read back from its text.

use std::cmp::Ordering;
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::entities::{FilePath, OutputMarker};
use crate::policies::rendering::{GHG_VERSION, STORE_FORMAT, json_bytes};

#[derive(Serialize, Deserialize)]
struct MarkerData {
    format: u32,
    ghg: String,
    files: Vec<FilePath>,
}

/// The marker's bytes, listing `files`, every file ghg wrote in the output folder, in the order
/// of every list of paths.
pub(crate) fn marker_json(files: &BTreeSet<FilePath>) -> Vec<u8> {
    let mut files: Vec<FilePath> = files.iter().cloned().collect();
    files.sort_by(|one, other| folder_by_folder(one, other));
    json_bytes(&MarkerData {
        format: STORE_FORMAT,
        ghg: GHG_VERSION.to_string(),
        files,
    })
}

/// What the marker `text` records; `None` when it is not a marker, or lists a path that leads
/// outside the output folder.
pub(crate) fn read_marker(text: &str) -> Option<OutputMarker> {
    let value: Value = serde_json::from_str(text).ok()?;
    if value.as_object()?.get("format").is_none() {
        return Some(OutputMarker::GherkinGlassOne);
    }
    let data: MarkerData = serde_json::from_value(value).ok()?;
    data.files
        .iter()
        .all(|file| is_inside(file))
        .then(|| OutputMarker::Store {
            format: data.format,
            ghg: data.ghg,
            files: data.files.into_iter().collect(),
        })
}

/// Paths compared folder by folder, as every list of paths is sorted.
pub(crate) fn folder_by_folder(one: &str, other: &str) -> Ordering {
    one.split('/').cmp(other.split('/'))
}

/// Whether `path` is a path inside the output folder: names joined by `/`, none of them empty,
/// `.` or `..`.
fn is_inside(path: &str) -> bool {
    path.split('/').all(|name| !matches!(name, "" | "." | ".."))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test_support::set;

    // Rules 245 and 247: the format, the ghg, then the files, folder by folder.
    #[test]
    fn a_marker_lists_its_files_folder_by_folder_and_reads_back() {
        let files = set(&["a b/x.json", "a/x.json", "index.html"]);
        let written = String::from_utf8(marker_json(&files)).expect("UTF-8");
        assert_eq!(
            written,
            format!(
                r#"{{"format":1,"ghg":"{GHG_VERSION}","files":["a/x.json","a b/x.json","index.html"]}}"#
            )
        );
        assert_eq!(
            read_marker(&written),
            Some(OutputMarker::Store {
                format: 1,
                ghg: GHG_VERSION.to_string(),
                files,
            })
        );
    }

    // Rule 256: a marker of gherkin-glass 1 holds no format.
    #[test]
    fn a_marker_without_a_format_is_gherkin_glass_1_s() {
        assert_eq!(
            read_marker("{\n  \"files\": [\"index.html\"]\n}"),
            Some(OutputMarker::GherkinGlassOne)
        );
    }

    // Rule 38: a marker that cannot be read, or leads outside the folder, counts as none.
    #[rstest]
    #[case::not_json("not a marker")]
    #[case::not_an_object("[1]")]
    #[case::no_files(r#"{"format":1,"ghg":"2.0.0"}"#)]
    #[case::outside(r#"{"format":1,"ghg":"2.0.0","files":["../notes.txt"]}"#)]
    #[case::an_empty_name(r#"{"format":1,"ghg":"2.0.0","files":["a//b"]}"#)]
    #[case::a_dot(r#"{"format":1,"ghg":"2.0.0","files":["./a"]}"#)]
    fn a_marker_that_cannot_be_read_is_none(#[case] text: &str) {
        assert_eq!(read_marker(text), None);
    }
}
