//! The page of step sentences, compared with the one committed beside each fixture.
//!
//! `ghg build --input . --title Fixture --output <folder>`, run in the
//! fixture's folder, writes the page again after a change, to be read before
//! it is committed.

#![expect(
    clippy::expect_used,
    reason = "a test file's helpers are test code, which clippy.toml lets panic"
)]

use std::fs::{read_dir, read_to_string, remove_dir_all};
use std::path::Path;
use std::process::Command;

/// The page of step sentences `ghg build` writes for the fixture `name`,
/// and the one committed beside it.
fn pages(name: &str) -> (String, String) {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let written = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("sentences-{name}"));
    let _ = remove_dir_all(&written);
    let result = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .args(["build", "--input", ".", "--title", "Fixture", "--output"])
        .arg(&written)
        .current_dir(&fixture)
        .output()
        .expect("the command runs");
    assert_eq!(String::from_utf8_lossy(&result.stderr), "");
    assert_eq!(result.status.code(), Some(0));
    (
        read_to_string(written.join("steps.html")).expect("the page is written"),
        read_to_string(fixture.join("expected/steps.html")).expect("the page is committed"),
    )
}

#[test]
fn the_page_shows_every_sentence_under_its_heading_with_links_to_its_uses() {
    // Rules 142 to 144.
    let (written, expected) = pages("step-sentences");
    assert!(written == expected, "steps.html differs:\n{written}");
}

#[test]
fn a_suite_with_no_step_has_a_page_that_says_so() {
    // Rule 145.
    let (written, expected) = pages("no-steps");
    assert!(written == expected, "steps.html differs:\n{written}");
}

#[test]
fn a_background_s_steps_are_listed_once_whether_the_box_is_checked_or_not() {
    // Rule 140: the backgrounds fixture's two sites, written with the checkbox checked and unchecked.
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backgrounds");
    let checked = read_to_string(fixture.join("expected/steps.html")).expect("committed");
    let unchecked =
        read_to_string(fixture.join("expected-unchecked/steps.html")).expect("committed");
    assert!(
        checked == unchecked,
        "the two sites list the same sentences"
    );
    let background = "<h3>a limit of 50</h3>\n<p>Used once.</p>\n<ul>\n\
                      <li><a href=\"both.html#over-the-limit\">";
    assert!(checked.contains(background), "{checked}");
}

#[test]
fn only_the_home_page_and_the_page_itself_link_the_page_of_step_sentences() {
    // Rule 146, in the pages fixture's site.
    let site = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pages/expected");
    let home = read_to_string(site.join("index.html")).expect("committed");
    assert!(
        home.contains(
            "</ul>\n<p class=\"ghg-tag-index-link\"><a href=\"tags/index.html\">Tags</a></p>\n\
             <p class=\"ghg-sentences\"><a href=\"steps.html\">Step sentences</a></p>\n<h2>"
        ),
        "{home}"
    );
    let mut linking = Vec::new();
    let mut folders = vec![site.clone()];
    while let Some(folder) = folders.pop() {
        for entry in read_dir(&folder).expect("listed") {
            let path = entry.expect("listed").path();
            if path.is_dir() {
                folders.push(path);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "html")
                && read_to_string(&path)
                    .expect("read")
                    .contains("steps.html\"")
            {
                let name = path.strip_prefix(&site).expect("inside");
                linking.push(name.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    linking.sort();
    assert_eq!(linking, ["index.html", "steps.html"]);
}
