//! docs/BEHAVIOUR.md, "Tags", checked by building the fixtures and comparing
//! every file written with the site committed beside them.
//!
//! `cargo run -- build --config tests/fixtures/tags/ghg.toml` writes the
//! expected site again after a change.

use std::collections::BTreeMap;
use std::fs::{read, read_dir, remove_dir_all};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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

/// Every file below `folder`, by its `/`-joined path in it.
fn files(folder: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut found = BTreeMap::new();
    let mut folders = vec![(folder.to_path_buf(), String::new())];
    while let Some((path, prefix)) = folders.pop() {
        for entry in read_dir(&path).expect("the folder is listed") {
            let entry = entry.expect("the folder is listed");
            let name = format!("{prefix}{}", entry.file_name().to_string_lossy());
            if entry.path().is_dir() {
                folders.push((entry.path(), format!("{name}/")));
            } else {
                found.insert(name, read(entry.path()).expect("the file is read"));
            }
        }
    }
    found
}

fn output(name: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = remove_dir_all(&folder);
    folder
}

fn text(file: &Path) -> String {
    String::from_utf8(read(file).expect("read")).expect("UTF-8")
}

// Rules 100 and 102 to 107: every page of the fixture, the tag pages among them.

#[test]
fn every_tag_has_its_page_and_every_page_links_its_tags_as_the_expected_site_shows() {
    let written = output("tags");
    let given = written.to_str().expect("UTF-8");
    let result = build(&fixtures().join("tags"), &["--output", given]);
    assert_eq!(String::from_utf8_lossy(&result.stderr), "");
    assert_eq!(result.status.code(), Some(0));
    let expected = files(&fixtures().join("tags/expected"));
    let written = files(&written);
    assert_eq!(
        written.keys().collect::<Vec<_>>(),
        expected.keys().collect::<Vec<_>>(),
        "the files written"
    );
    for (name, contents) in &expected {
        assert!(
            written[name] == *contents,
            "{name} differs from the expected file:\n{}",
            String::from_utf8_lossy(&written[name])
        );
    }
}

#[test]
fn the_tag_pages_are_in_english_with_one_h1() {
    for (name, page) in files(&fixtures().join("tags/expected/tags")) {
        let page = String::from_utf8(page).expect("UTF-8");
        assert_eq!(page.matches("<h1").count(), 1, "{name}");
        assert!(page.contains("<html lang=\"en\">"), "{name}");
    }
}

// Rule 102, with no tag at all.

#[test]
fn a_site_without_tags_has_an_index_that_says_so() {
    let written = output("no-tags");
    let given = written.to_str().expect("UTF-8");
    let result = build(
        &fixtures().join("configuration/default"),
        &["--output", given],
    );
    assert_eq!(result.status.code(), Some(0));
    let index = text(&written.join("tags/index.html"));
    assert!(
        index.contains("<h1>Tags</h1>\n<p>No feature file has a tag.</p>\n"),
        "{index}"
    );
    let tags: Vec<_> = files(&written.join("tags")).into_keys().collect();
    assert_eq!(tags, ["index.html"]);
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
        "tags/tagging.feature: has its page, tags/tagging.html, in the folder tags, \
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
