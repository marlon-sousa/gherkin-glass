//! A build that reads no feature file, checked by running `ghg build` in
//! `tests/fixtures/no-features`, whose input holds a file that is not a feature file and a feature
//! file its `.ghg-ignore` leaves out.

#![expect(
    clippy::expect_used,
    reason = "a test file's helpers are test code, which clippy.toml lets panic"
)]

use std::fs::{create_dir_all, read_dir, remove_dir_all, write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rstest::rstest;

#[path = "support/site_files.rs"]
#[expect(
    dead_code,
    reason = "this file lists what a folder holds, and compares no site with an expected one"
)]
mod site_files;
#[path = "support/version_placeholder.rs"]
mod version_placeholder;

use site_files::files;

/// An output folder of its own for `test`, which does not exist yet.
fn output_folder(test: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("nothing-to-build")
        .join(test);
    let _ = remove_dir_all(&folder);
    folder
}

/// `ghg build` run in the fixture, writing into `output`, with `arguments`.
fn build(output: &Path, arguments: &[&str]) -> Output {
    ghg("build", output, arguments)
}

/// `ghg <command>` run in the fixture, writing into `output`, with `arguments`.
fn ghg(command: &str, output: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ghg"))
        .arg(command)
        .arg("--output")
        .arg(output)
        .args(arguments)
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/no-features"))
        .output()
        .expect("the command runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

// Rule 229: nothing is written, not even the output folder, whether the build is versioned or a
// preview, and the build says why and is done.
#[rstest]
#[case::a_site(&[], "site")]
#[case::a_version(&["--version-name", "v1.0.0"], "version")]
#[case::a_preview(&["--preview"], "preview")]
fn a_build_that_reads_no_feature_file_writes_nothing_and_says_why(
    #[case] arguments: &[&str],
    #[case] test: &str,
) {
    let output = output_folder(test);
    let run = build(&output, arguments);
    assert_eq!(
        text(&run.stderr),
        "features: the input folder holds no feature file, so no site was built.\n"
    );
    assert_eq!(text(&run.stdout), "");
    assert_eq!(run.status.code(), Some(0));
    assert!(!output.exists());
}

// Rules 38 and 229: the output folder is checked first, so one ghg did not write still stops the
// build, and is left as it was.
#[test]
fn an_output_folder_ghg_did_not_write_still_stops_a_build_that_reads_no_feature_file() {
    let output = output_folder("not-a-site");
    create_dir_all(&output).expect("made");
    write(output.join("notes.txt"), "mine\n").expect("written");
    let run = build(&output, &[]);
    assert_eq!(run.status.code(), Some(1));
    assert!(
        text(&run.stderr).ends_with(
            "this is not a site ghg built, so it is left as it is. Build into an empty or new folder.\n"
        ),
        "{}",
        text(&run.stderr)
    );
    let left: Vec<_> = read_dir(&output)
        .expect("listed")
        .map(|entry| entry.expect("an entry").file_name())
        .collect();
    assert_eq!(left, ["notes.txt"]);
}

// Rule 229: an output folder that already holds versions is left as it was, its versions page
// included.
#[test]
fn a_version_that_reads_no_feature_file_leaves_the_versions_written_as_they_were() {
    let output = output_folder("versions-kept");
    let constructs = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/constructs");
    let first = build(
        &output,
        &[
            "--input",
            &constructs.to_string_lossy(),
            "--version-name",
            "v1",
        ],
    );
    assert_eq!(first.status.code(), Some(0), "{}", text(&first.stderr));
    let before = files(&output);
    let run = build(&output, &["--version-name", "v2"]);
    assert_eq!(run.status.code(), Some(0));
    assert_eq!(files(&output), before);
}

// Rule 230: ghg serve, building into the output folder, writes nothing and serves nothing.
#[test]
fn serve_that_reads_no_feature_file_writes_and_serves_nothing() {
    let output = output_folder("serve");
    let run = ghg("serve", &output, &["--port", "0"]);
    assert_eq!(
        text(&run.stderr),
        "features: the input folder holds no feature file, so no site was built.\n"
    );
    assert_eq!(text(&run.stdout), "");
    assert_eq!(run.status.code(), Some(0));
    assert!(!output.exists());
}
