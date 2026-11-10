//! docs/BEHAVIOUR.md, "Search", checked against the fixtures.

use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use gherkin_glass::{
    GherkinParser, InputService, InputWalker, Unnamed, make_anchors, search_documents, search_json,
};
use serde_json::Value;

fn fixture() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/search"
    ))
}

fn documents() -> String {
    let features = InputService::new(Arc::new(InputWalker), Arc::new(GherkinParser))
        .read(&fixture().join("features"), Unnamed::Allowed)
        .unwrap_or_else(|problems| panic!("the input is read:\n{problems}"));
    let anchors: Vec<_> = features
        .iter()
        .map(|parsed| make_anchors(&parsed.feature))
        .collect();
    search_json(&search_documents(features.iter().zip(&anchors)))
}

#[test]
fn every_feature_rule_and_scenario_has_its_document_in_page_and_file_order() {
    // Rules 147 and 148.
    let expected = fs::read_to_string(fixture().join("search.json")).unwrap();
    let expected: Value = serde_json::from_str(&expected).unwrap();
    let written: Value = serde_json::from_str(&documents()).unwrap();
    assert_eq!(written, expected);
}

#[test]
fn the_documents_are_compact_and_keep_letters_outside_ascii() {
    // Rule 149.
    let written = documents();
    assert!(!written.contains('\n'));
    assert!(!written.contains("\": "));
    assert!(!written.contains("\", "));
    assert!(written.contains("poupança"));
    assert!(!written.contains("\\u"));
}

#[test]
fn the_documents_are_the_same_on_every_build() {
    // Rule 149.
    assert_eq!(documents(), documents());
}

#[test]
fn every_page_has_the_search_box_before_its_main() {
    // Rules 86 and 151.
    let pages = fixture().parent().unwrap().join("pages");
    for page in [
        "expected/billing/refunds.html",
        "expected/emoji.html",
        "expected/stories/read.html",
        "expected/saque em conta poupança.html",
        "expected-collapsed/billing/refunds.html",
    ] {
        let html = fs::read_to_string(pages.join(page)).expect(page);
        let form = html
            .find("<form class=\"ghg-search\" role=\"search\"")
            .expect(page);
        let main = html.find("<main id=\"ghg_main\">").expect(page);
        let breadcrumb = html.find("<nav class=\"ghg-breadcrumb\"").expect(page);
        assert!(breadcrumb < form && form < main, "{page}");
        let search = &html[form..main];
        assert_eq!(html.matches("role=\"search\"").count(), 1, "{page}");
        assert!(
            search.contains("<label>Search features, rules and scenarios <input type=\"search\"")
        );
        assert!(search.contains("<p class=\"ghg-search-status\" role=\"status\"></p>"));
        assert!(search.contains("<ol class=\"ghg-search-results\" hidden></ol>"));
    }
}

#[test]
fn documents_above_5_mb_are_written_with_a_warning() {
    // Rule 150, after the hint of rule 17.
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join("large-search");
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(folder.join("features")).unwrap();
    let step = "a".repeat(5_050_000);
    fs::write(
        folder.join("features/large.feature"),
        format!("Feature: Large\n  Scenario: Long\n    Given {step}\n"),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .arg("build")
        .current_dir(&folder)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "The site has no title, so its home page is called Feature files. Set title in ghg.toml, or give --title.\n\
         The search documents are 5.1 MB, larger than the 5 MB above which search can be slow to start, mostly on phones.\n"
    );
    let written = fs::metadata(folder.join("site/assets/search.json")).unwrap();
    assert!(written.len() > 5_000_000);
}

#[test]
fn documents_of_5_mb_or_less_bring_no_warning() {
    // Rule 150, and rule 17: a build that succeeds prints nothing else.
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join("small-search");
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .args(["build", "--title", "Search", "--input"])
        .arg(fixture().join("features"))
        .current_dir(&folder)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
    let written = fs::read_to_string(folder.join("site/assets/search.json")).unwrap();
    assert_eq!(written, documents());
}
