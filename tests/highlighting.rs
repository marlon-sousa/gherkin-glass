//! Highlighting and themes: the fixture's site compared with the one committed beside it.
//!
//! `cargo xtask build` writes the expected site again after a change.

#![expect(
    clippy::expect_used,
    reason = "a test file's helpers are test code, which clippy.toml lets panic"
)]

use std::fs::{read_to_string, remove_dir_all};
use std::path::Path;
use std::process::Command;

#[path = "support/site_files.rs"]
mod site_files;
#[path = "support/store_json.rs"]
mod store_json;
#[path = "support/version_placeholder.rs"]
mod version_placeholder;

use site_files::assert_same_site;

// Rules 109 and 118: each part of a step in the store, and the configured
// accents in the shell's head.

#[test]
fn the_site_holds_the_steps_parts_and_the_accents_as_the_expected_site_does() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/highlighting");
    let written = Path::new(env!("CARGO_TARGET_TMPDIR")).join("highlighting");
    let _ = remove_dir_all(&written);
    let result = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .args(["build", "--output", written.to_str().expect("UTF-8")])
        .current_dir(&fixture)
        .output()
        .expect("the command runs");
    assert_eq!(String::from_utf8_lossy(&result.stderr), "");
    assert_eq!(result.status.code(), Some(0));
    assert_same_site(&written, &fixture.join("expected"));
    let shell = read_to_string(written.join("index.html")).expect("the shell is written");
    assert!(shell.contains("  --ghg-accent: #5a1e8c;\n"));
    assert!(shell.contains("  --ghg-accent: #ffcc88;\n"));
}
