//! A site of only the feature files a list names, checked by building `tests/fixtures/only`
//! with `--only`, adding it as a version and as a preview, and comparing what is written with the
//! sites committed beside it.

#![expect(
    clippy::expect_used,
    reason = "a test file's helpers are test code, which clippy.toml lets panic"
)]

use std::fs::{read, read_to_string, remove_dir_all};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use rstest::rstest;

#[path = "support/site_files.rs"]
mod site_files;
#[path = "support/store_json.rs"]
mod store_json;
#[path = "support/version_placeholder.rs"]
mod version_placeholder;

use site_files::assert_same_site;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/only")
}

/// `ghg build` run in the fixture with `arguments`, `standard_input` given to it, writing into
/// a fresh folder named `name`, which it returns with the run's output.
fn build(arguments: &[&str], standard_input: &str, name: &str) -> (PathBuf, Output) {
    let output = output_folder(name);
    let _ = remove_dir_all(&output);
    let run = ghg(&[&["build"], arguments].concat(), standard_input, &output);
    (output, run)
}

fn output_folder(name: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("only")
        .join(name)
}

/// `ghg` run in the fixture with `arguments`, `standard_input` given to it, writing into
/// `output`.
fn ghg(arguments: &[&str], standard_input: &str, output: &Path) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .args(arguments)
        .arg("--output")
        .arg(output)
        .current_dir(fixture())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the command runs");
    child
        .stdin
        .take()
        .expect("standard input")
        .write_all(standard_input.as_bytes())
        .expect("standard input is written");
    child.wait_with_output().expect("the command ends")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn changed() -> String {
    text(&read(fixture().join("changed.txt")).expect("the list is read"))
}

// Rules 204 to 207: the files the list names that the input folder and its rules read, alone,
// each page saying so, and nothing reported of the files left out, such as one that is not
// Gherkin.
#[rstest]
#[case::from_a_file(&["--only", "changed.txt"], String::new(), "file")]
#[case::from_standard_input(&["--only", "-"], changed(), "standard-input")]
fn the_site_holds_only_the_listed_files_it_reads(
    #[case] arguments: &[&str],
    #[case] standard_input: String,
    #[case] name: &str,
) {
    let (written, run) = build(arguments, &standard_input, name);
    assert_eq!(text(&run.stderr), "");
    assert_eq!(run.status.code(), Some(0));
    assert_same_site(&written, &fixture().join("expected"));
}

// Rules 208 and 229: a list none of whose files is read builds nothing, and says why.
#[test]
fn a_list_none_of_whose_files_is_read_builds_nothing() {
    let (written, run) = build(
        &["--config", "nothing.toml", "--only", "nothing.txt"],
        "",
        "nothing",
    );
    assert_eq!(
        text(&run.stderr),
        "nothing.txt: names no feature file of the input, so no site was built.\n"
    );
    assert_eq!(text(&run.stdout), "");
    assert_eq!(run.status.code(), Some(0));
    assert!(!written.exists());
}

// Rule 205: a listed file that is read but cannot be parsed fails the build as rule 14 says.
#[test]
fn a_listed_file_that_cannot_be_parsed_stops_the_build() {
    let (written, run) = build(
        &["--only", "-"],
        "features/billing/broken.feature\n",
        "broken",
    );
    assert!(
        text(&run.stderr).starts_with("billing/broken.feature:1: "),
        "{}",
        text(&run.stderr)
    );
    assert_eq!(run.status.code(), Some(1));
    assert!(!written.exists());
}

// Rule 204.
#[test]
fn a_list_that_cannot_be_read_stops_the_run_before_the_input_is_read() {
    let (written, run) = build(&["--only", "missing.txt"], "", "missing");
    assert!(
        text(&run.stderr).starts_with("missing.txt: cannot be read: "),
        "{}",
        text(&run.stderr)
    );
    assert_eq!(text(&run.stdout), "");
    assert_eq!(run.status.code(), Some(1));
    assert!(!written.exists());
}

// Rules 204 and 229: a list that names no path builds nothing, and says why.
#[rstest]
#[case::a_file_of_empty_lines(&["--only", "empty.txt"], "", "empty.txt")]
#[case::nothing_on_standard_input(&["--only", "-"], "", "-")]
fn a_list_that_names_no_path_builds_nothing(
    #[case] arguments: &[&str],
    #[case] standard_input: &str,
    #[case] named: &str,
) {
    let (written, run) = build(arguments, standard_input, "empty");
    assert_eq!(
        text(&run.stderr),
        format!("{named}: names no path, so there is nothing to build.\n")
    );
    assert_eq!(text(&run.stdout), "");
    assert_eq!(run.status.code(), Some(0));
    assert!(!written.exists());
}

// Rule 204: serving without building has no use for a list.
#[test]
fn a_list_given_with_no_build_is_a_mistake_on_the_command_line() {
    let run = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .args(["serve", "--no-build", "--only", "changed.txt"])
        .current_dir(fixture())
        .output()
        .expect("the command runs");
    assert_eq!(
        text(&run.stderr),
        "The options --only and --no-build cannot be used together.\n"
    );
    assert_eq!(text(&run.stdout), "");
    assert_eq!(run.status.code(), Some(2));
}

// Rule 210.
#[test]
fn a_version_of_only_the_listed_files_is_listed_as_any_other_and_says_what_it_shows() {
    let output = output_folder("version");
    let _ = remove_dir_all(&output);
    let run = ghg(
        &["version", "add", "pr-131", "--only", "changed.txt"],
        "",
        &output,
    );
    assert_eq!(text(&run.stderr), "");
    assert_eq!(run.status.code(), Some(0));
    let versions = read_to_string(output.join("index.html")).expect("the versions page");
    assert!(
        versions.contains("href=\"pr-131/index.html\""),
        "{versions}"
    );
    assert!(!versions.contains("given with --only"), "{versions}");
    let descriptor =
        read_to_string(output.join("data/versions/pr-131.json")).expect("the descriptor");
    assert!(descriptor.contains("\"only\":true"), "{descriptor}");
}

// Rules 211, 213 and 258: a preview of the listed files, added beside the
// site and removed again, which leaves the site as it was.
#[test]
fn a_preview_is_added_beside_the_site_and_removed_from_it() {
    let (output, run) = build(&["--only", "changed.txt"], "", "preview");
    assert_eq!(run.status.code(), Some(0));
    let run = ghg(
        &["preview", "add", "pr-131", "--only", "changed.txt"],
        "",
        &output,
    );
    assert_eq!(text(&run.stderr), "");
    assert_eq!(run.status.code(), Some(0));
    assert_same_site(&output, &fixture().join("expected-preview"));
    let shell = read_to_string(output.join("previews/pr-131/index.html")).expect("the shell");
    assert!(shell.contains("</title>\n<meta name=\"robots\" content=\"noindex\">"));
    let run = ghg(&["preview", "remove", "pr-131"], "", &output);
    assert_eq!(text(&run.stderr), "");
    assert_eq!(run.status.code(), Some(0));
    assert_same_site(&output, &fixture().join("expected"));
}

// Rule 124: a preview named as another but for its capitals stops the run.
#[test]
fn a_preview_named_as_another_but_for_its_capitals_stops_the_run() {
    let output = output_folder("preview-capitals");
    let _ = remove_dir_all(&output);
    let run = ghg(
        &["preview", "add", "pr-131", "--only", "changed.txt"],
        "",
        &output,
    );
    assert_eq!(run.status.code(), Some(0));
    let run = ghg(
        &["preview", "add", "PR-131", "--only", "changed.txt"],
        "",
        &output,
    );
    assert_eq!(
        text(&run.stderr),
        format!(
            "{}/previews/PR-131: the output folder holds a preview whose name differs from this one only in its capitals, which some systems would put in the same folder, so it is left as it is.\n",
            output.display()
        )
    );
    assert_eq!(run.status.code(), Some(1));
}
