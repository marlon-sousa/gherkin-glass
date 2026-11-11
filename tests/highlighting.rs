//! Highlighting and themes: the fixture's pages compared with those committed beside it.
//!
//! `cargo xtask build` writes the expected pages again after a change.

use std::fs::{read, read_dir, remove_dir_all};
use std::path::Path;
use std::process::Command;

// Rules 109 and 118: each part in its own element, and the configured accents in the page's head.

#[test]
fn the_pages_show_the_highlighting_and_the_accents_as_the_expected_pages_do() {
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
    let expected = fixture.join("expected");
    let mut pages = 0;
    for entry in read_dir(&expected).expect("the expected pages are listed") {
        let name = entry.expect("listed").file_name();
        if !name.to_string_lossy().ends_with(".html") {
            continue;
        }
        let page = read(written.join(&name)).expect("the page is written");
        assert!(
            page == read(expected.join(&name)).expect("read"),
            "{name:?} differs from the expected page:\n{}",
            String::from_utf8_lossy(&page)
        );
        let page = String::from_utf8_lossy(&page);
        assert!(page.contains("  --ghg-accent: #5a1e8c;\n"), "{name:?}");
        assert!(page.contains("  --ghg-accent: #ffcc88;\n"), "{name:?}");
        pages += 1;
    }
    assert_eq!(pages, 4);
}
