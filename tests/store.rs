//! The site's data, checked by building the fixtures whose store is committed without their
//! site, and comparing the store written with theirs, `expected-store`, and the stores of
//! `tests/fixtures/store` that no command writes.
//!
//! `cargo xtask build` writes the expected stores again after a change.

#![expect(
    clippy::expect_used,
    reason = "a test file's helpers are test code, which clippy.toml lets panic"
)]

use std::fs::remove_dir_all;
use std::path::{Path, PathBuf};
use std::process::Command;

use rstest::rstest;

#[path = "support/copied_folder.rs"]
mod copied_folder;
#[path = "support/site_files.rs"]
#[expect(
    dead_code,
    reason = "this file compares only the store of a site with an expected one"
)]
mod site_files;
#[path = "support/store_json.rs"]
mod store_json;
#[path = "support/version_placeholder.rs"]
mod version_placeholder;

use copied_folder::copy_folder;
use site_files::{assert_same_store, files};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

// Rule 248: every construct. Rule 249: anchors. Rule 250: doc strings and step sentences.
// Rule 252: a configuration.
#[rstest]
#[case::constructs("constructs", &["--input", "."])]
#[case::anchors("anchors", &["--input", "."])]
#[case::doc_strings("doc-strings", &["--input", "."])]
#[case::step_sentences("step-sentences", &["--input", "."])]
#[case::configured("configuration/configured", &[])]
fn a_fixture_s_store_is_its_committed_copy(#[case] name: &str, #[case] arguments: &[&str]) {
    let output = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("stores")
        .join(name);
    let _ = remove_dir_all(&output);
    let result = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .arg("build")
        .arg("--output")
        .arg(&output)
        .args(arguments)
        .current_dir(fixture(name))
        .output()
        .expect("the command runs");
    assert_eq!(result.status.code(), Some(0), "{result:?}");
    assert_same_store(&output, &fixture(name).join("expected-store"));
}

/// A fresh copy of the fixture `store/<name>`, for a test that tries to write it.
fn store_copy(name: &str, test: &str) -> PathBuf {
    let copied = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("stores-refused")
        .join(test);
    let _ = remove_dir_all(&copied);
    copy_folder(&fixture("store").join(name), &copied);
    copied
}

/// `ghg` with `arguments`, writing into `output`, run in the suite of `tests/fixtures/versions`:
/// what it says on standard error, and its status.
fn ghg(arguments: &[&str], output: &Path) -> (String, i32) {
    let result = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .args(arguments)
        .arg("--output")
        .arg(output)
        .current_dir(fixture("versions/suite"))
        .output()
        .expect("the command runs");
    (
        String::from_utf8_lossy(&result.stderr).into_owned(),
        result.status.code().expect("a status"),
    )
}

/// Every command that writes an output folder.
const WRITING: [&[&str]; 5] = [
    &["build"],
    &["version", "add", "v3.0.0"],
    &["version", "remove", "main"],
    &["preview", "add", "pr-1"],
    &["preview", "remove", "pr-1"],
];

// Rule 256: an output folder of gherkin-glass 1 stops every command that writes it, which
// writes and deletes nothing.
#[test]
fn an_output_folder_of_gherkin_glass_1_is_left_as_it_is() {
    for (place, arguments) in WRITING.iter().enumerate() {
        let output = store_copy("version-1", &format!("version-1-{place}"));
        let report = ghg(arguments, &output);
        assert_eq!(
            report,
            (
                format!(
                    "{}: a site of gherkin-glass 1 is built here, which gherkin-glass 2 cannot add to, so it is left as it is. Build into an empty or new folder.\n",
                    output.display()
                ),
                1
            ),
            "{arguments:?}"
        );
        assert_eq!(files(&output), files(&fixture("store/version-1")));
    }
}

// Rule 246: a store in a newer format stops every command that writes it, which writes and
// deletes nothing.
#[test]
fn a_store_in_a_newer_format_is_left_as_it_is() {
    for (place, arguments) in WRITING.iter().enumerate() {
        let output = store_copy("newer", &format!("newer-{place}"));
        let report = ghg(arguments, &output);
        assert_eq!(
            report,
            (
                format!(
                    "{}: the site's data was written by ghg 3.0.0, in a newer format than this ghg reads, so it is left as it is. Use ghg 3.0.0 or later.\n",
                    output.display()
                ),
                1
            ),
            "{arguments:?}"
        );
        assert_eq!(files(&output), files(&fixture("store/newer")));
    }
}
