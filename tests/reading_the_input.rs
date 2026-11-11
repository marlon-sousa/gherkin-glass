//! Reading the input: the report of files that cannot be read, by running `ghg`.

use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

// Files that cannot be read: rules 14 to 16.

#[test]
fn every_file_that_cannot_be_read_is_reported_sorted_by_path() {
    // The report as `ghg build` prints it, in the catalogue's words.
    let output = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .arg("build")
        .arg("--input")
        .arg(fixture("unreadable"))
        .output()
        .expect("the command runs");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        String::from_utf8(output.stderr).expect("the report is UTF-8"),
        [
            "empty.feature:1: Syntax error.",
            "folder/.ghg-ignore:2: error parsing glob 'a{b': unclosed alternate group; missing '}' (maybe escape '{' with '[{]'?)",
            "folder/uneven-table.feature:5: Inconsistent cell count",
            "not-utf-8.feature: cannot be read: stream did not contain valid UTF-8",
            "unknown-keyword.feature:4: Unknown keyword: 'Blah'.",
            "unknown-language.feature:2: Requested language 'xx-unknown' is not supported.",
            "",
        ]
        .join("\n")
    );
}
