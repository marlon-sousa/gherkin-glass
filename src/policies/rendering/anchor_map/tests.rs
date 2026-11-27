use super::*;
use crate::entities::{PageOptions, ParsedFeature};
use crate::policies::{build_store, examples_row_lines};
use crate::test_support::parsed_at;

/// The map of the features of `files`, each a path and its text, read as the parser reads them.
fn map(files: &[(&str, &str)]) -> serde_json::Value {
    let parsed: Vec<ParsedFeature> = files
        .iter()
        .map(|(path, text)| {
            let mut parsed = parsed_at(path, text);
            parsed.row_lines = examples_row_lines(&parsed.feature, text);
            parsed
        })
        .collect();
    let features: Vec<&ParsedFeature> = parsed.iter().collect();
    let bytes = anchor_map_json(&build_store(&features, &PageOptions::default()));
    serde_json::from_slice(&bytes).expect("the map is JSON")
}

// Rules 264 and 265: an entry for each block, in file order, each with its path, kind, name,
// line, page and anchor; an examples block's rows with their own lines, past a comment and a
// blank line.
#[test]
fn every_block_has_its_line_page_and_anchor_and_every_row_its_own() {
    let text = "Feature: Lend\n\
                \x20 Background:\n\
                \x20   Given a library\n\
                \x20 Scenario: Borrow\n\
                \x20   Given a book\n\
                \x20 Scenario: Borrow\n\
                \x20   Given another\n\
                \x20 Scenario Outline: Return\n\
                \x20   Given <n> days\n\
                \x20   Examples:\n\
                \x20     | n |\n\
                \x20     | 1 |\n\
                \x20     # A comment.\n\
                \n\
                \x20     | 2 |\n\
                \x20 Rule:\n\
                \x20   Scenario: Renew\n\
                \x20     Given a loan\n";
    let entry = |kind: &str, name: &str, line: usize, anchor: &str| serde_json::json!({"path": "lending/lend.feature", "kind": kind, "name": name, "line": line, "page": "lending/lend", "anchor": anchor, "rows": null});
    assert_eq!(
        map(&[("lending/lend.feature", text)]),
        serde_json::json!([
            entry("feature", "Lend", 1, ""),
            entry("background", "", 2, "background"),
            entry("scenario", "Borrow", 4, "borrow"),
            entry("scenario", "Borrow", 6, "borrow-2"),
            entry("scenario-outline", "Return", 8, "return"),
            {"path": "lending/lend.feature", "kind": "examples", "name": "", "line": 10, "page": "lending/lend", "anchor": "examples",
             "rows": [{"line": 12, "anchor": "return-row-1"}, {"line": 15, "anchor": "return-row-2"}]},
            entry("rule", "", 16, "rule"),
            entry("scenario", "Renew", 17, "renew"),
        ])
    );
}

// Rule 264: the files in the order of rule 15, and an examples block without rows has an empty
// list.
#[test]
fn files_follow_their_order_and_rows_may_be_none() {
    let map = map(&[
        ("b.feature", "Feature: B\n"),
        (
            "a.feature",
            "Feature: A\n  Scenario Outline: O\n    Given <x>\n    Examples:\n      | x |\n",
        ),
    ]);
    let paths: Vec<&str> = map
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry["path"].as_str())
        .collect();
    assert_eq!(paths, ["a.feature", "a.feature", "a.feature", "b.feature"]);
    assert_eq!(map[2]["rows"], serde_json::json!([]));
}
