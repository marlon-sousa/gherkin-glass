//! The step sentences a build lists in the site's descriptor, and the front
//! page's link to their page.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "a test file's helpers are test code, which clippy.toml lets panic"
)]

use std::fs::{read_to_string, remove_dir_all};
use std::path::Path;
use std::process::Command;

use serde_json::{Value, json};

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// The sentences of the descriptor of the site in `site`.
fn sentences(site: &Path) -> Value {
    let descriptor = read_to_string(site.join("data/site.json")).expect("the descriptor is read");
    let descriptor: Value = serde_json::from_str(&descriptor).expect("JSON");
    descriptor["sentences"].clone()
}

// Rule 145.
#[test]
fn a_suite_with_no_step_lists_no_sentence() {
    let written = Path::new(env!("CARGO_TARGET_TMPDIR")).join("sentences-no-steps");
    let _ = remove_dir_all(&written);
    let result = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .args(["build", "--input", ".", "--title", "Fixture", "--output"])
        .arg(&written)
        .current_dir(fixture("no-steps"))
        .output()
        .expect("the command runs");
    assert_eq!(String::from_utf8_lossy(&result.stderr), "");
    assert_eq!(result.status.code(), Some(0));
    assert_eq!(
        sentences(&written),
        json!({"given": [], "when": [], "then": []})
    );
}

// Rule 140: the backgrounds fixture's two sites, written with the checkbox
// checked and unchecked, list the same sentences, a background's among them.
#[test]
fn a_background_s_steps_are_listed_whether_the_box_is_checked_or_not() {
    let checked = sentences(&fixture("backgrounds/expected"));
    assert_eq!(
        checked,
        sentences(&fixture("backgrounds/expected-unchecked"))
    );
    assert_eq!(checked["given"][0], "a limit of 50");
}

// Rule 146: the front page links the page of step sentences after the index
// of tags, and the shell holds no other link to it.
#[test]
fn only_the_front_page_links_the_page_of_step_sentences() {
    let home = read_to_string(fixture("pages/expected/index.html")).expect("committed");
    let link = "</ul>\n<p class=\"ghg-tag-index-link\"><a href=\"#tags/\">Tags</a></p>\n\
                <p class=\"ghg-sentences\"><a href=\"#steps\">Step sentences</a></p>\n<h2>";
    assert!(home.contains(link), "{home}");
    assert_eq!(home.matches("href=\"#steps\"").count(), 1);
}
