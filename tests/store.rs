//! The site's data, checked by building the fixtures whose store is committed without their
//! site, and comparing the store written with theirs, `expected-store`.
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

use site_files::assert_same_store;

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
