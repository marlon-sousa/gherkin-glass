//! Tags, checked by building the fixtures and comparing every file written with the site
//! committed beside them.
//!
//! `cargo xtask build` writes the expected site again after a change.

#![expect(
    clippy::expect_used,
    reason = "a test file's helpers are test code, which clippy.toml lets panic"
)]

use std::fs::{read_to_string, remove_dir_all};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[path = "support/site_files.rs"]
mod site_files;
#[path = "support/store_json.rs"]
mod store_json;
#[path = "support/version_placeholder.rs"]
mod version_placeholder;

use serde_json::{Value, json};
use site_files::assert_same_site;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// `ghg build` run in `folder` with `arguments` after it.
fn build(folder: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ghg"))
        .arg("build")
        .args(arguments)
        .current_dir(folder)
        .output()
        .expect("the command runs")
}

fn output(name: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = remove_dir_all(&folder);
    folder
}

// Rules 100 and 102 to 107: the tags the descriptor lists, and their links.

#[test]
fn every_tag_has_its_page_and_every_page_links_its_tags_as_the_expected_site_shows() {
    let written = output("tags");
    let given = written.to_str().expect("UTF-8");
    let result = build(&fixtures().join("tags"), &["--output", given]);
    assert_eq!(String::from_utf8_lossy(&result.stderr), "");
    assert_eq!(result.status.code(), Some(0));
    assert_same_site(&written, &fixtures().join("tags/expected"));
}

// Rule 102, with no tag at all: the descriptor lists none, and the script's
// page of tags says so.

#[test]
fn a_site_without_tags_lists_none() {
    let written = output("no-tags");
    let given = written.to_str().expect("UTF-8");
    let result = build(
        &fixtures().join("configuration/default"),
        &["--output", given],
    );
    assert_eq!(result.status.code(), Some(0));
    let descriptor: Value =
        serde_json::from_str(&read_to_string(written.join("data/site.json")).expect("read"))
            .expect("JSON");
    assert_eq!(descriptor["tags"], json!([]));
}

// Rule 101.

#[test]
fn a_feature_whose_page_would_be_in_the_tags_folder_stops_the_build() {
    let written = output("tags-folder");
    let result = build(
        &fixtures().join("tags-folder"),
        &["--output", written.to_str().expect("UTF-8")],
    );
    assert_eq!(
        String::from_utf8_lossy(&result.stderr),
        "tags/tagging.feature: has its page, tags/tagging, in the folder tags, \
         which holds the tag pages.\n"
    );
    assert_eq!(result.status.code(), Some(1));
    assert!(!written.exists(), "nothing is written");
}

// Rule 108.

#[test]
fn every_mistake_in_tag_links_is_reported_by_line() {
    let folder = fixtures().join("configuration/mistaken-tags");
    let result = build(&folder, &[]);
    assert_eq!(
        String::from_utf8_lossy(&result.stderr),
        "ghg.toml:4: tag_links.pattern is not a regular expression: unclosed group\n\
         ghg.toml:10: tag_links.address names the group 2, which the pattern does not have.\n\
         ghg.toml:11: tag_links.text names the group 1a, which the pattern does not have.\n\
         ghg.toml:13: tag_links.address is missing.\n\
         ghg.toml:14: tag_links.pattern must be text, in quotes.\n\
         ghg.toml:15: ghg has no setting named tag_links.colour.\n\
         ghg.toml:16: tag_links.text must not be empty.\n\
         ghg.toml:18: tag_links.address is missing.\n\
         ghg.toml:18: tag_links.pattern is missing.\n\
         ghg.toml:18: tag_links.text is missing.\n\
         ghg.toml:20: tag_links.text is missing.\n\
         ghg.toml:21: tag_links.pattern is not a regular expression: unclosed group\n\
         ghg.toml:26: tag_links.address names the group 1, which the pattern does not have.\n\
         ghg.toml:27: tag_links.text must be text, in quotes.\n\
         ghg.toml:30: tag_links.pattern is too large a regular expression.\n"
    );
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    let result = build(&folder, &["--config", "list.toml"]);
    assert_eq!(
        String::from_utf8_lossy(&result.stderr),
        "list.toml:1: tag_links must be a list of tables, each written [[tag_links]].\n"
    );
    assert_eq!(result.status.code(), Some(1));
}
