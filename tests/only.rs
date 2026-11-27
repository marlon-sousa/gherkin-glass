//! A site of only the feature files a list names, checked by building `tests/fixtures/only`
//! with `--only`, and as a preview with `--preview`, and comparing what is written with the sites
//! committed beside it.

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

use site_files::{assert_same_site, files};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/only")
}

/// `ghg build` run in the fixture with `arguments`, `standard_input` given to it, writing into
/// a fresh folder named `name`, which it returns with the run's output.
fn build(arguments: &[&str], standard_input: &str, name: &str) -> (PathBuf, Output) {
    let output = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("only")
        .join(name);
    let _ = remove_dir_all(&output);
    let mut child = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .arg("build")
        .args(arguments)
        .arg("--output")
        .arg(&output)
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
    let run = child.wait_with_output().expect("the command ends");
    (output, run)
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
    let (written, run) = build(
        &["--only", "changed.txt", "--version-name", "pr-131"],
        "",
        "version",
    );
    assert_eq!(text(&run.stderr), "");
    assert_eq!(run.status.code(), Some(0));
    let versions = read_to_string(written.join("index.html")).expect("the versions page");
    assert!(
        versions.contains("href=\"pr-131/index.html\""),
        "{versions}"
    );
    assert!(!versions.contains("given with --only"), "{versions}");
    let page = read_to_string(written.join("pr-131/welcome.html")).expect("a page");
    assert!(page.contains("given with --only"), "{page}");
}

/// The robots element of rule 211, right after the page's title.
const NOINDEX: &str = "</title>\n<meta name=\"robots\" content=\"noindex\">";

/// `page`, an HTML page, without the robots element of rule 211.
fn indexed(page: &str) -> String {
    page.replace(NOINDEX, "</title>")
}

// Rules 211 and 213: a preview is the site without versions the build would make, whatever
// version the configuration names, every page of it asking not to be indexed, and nothing else
// changed.
#[test]
fn a_preview_is_the_site_without_versions_every_page_asking_not_to_be_indexed() {
    let (written, run) = build(
        &[
            "--config",
            "versioned.toml",
            "--only",
            "changed.txt",
            "--preview",
        ],
        "",
        "preview",
    );
    assert_eq!(text(&run.stderr), "");
    assert_eq!(run.status.code(), Some(0));
    let expected = files(&fixture().join("expected"));
    let preview = files(&written);
    assert_eq!(
        preview.keys().collect::<Vec<_>>(),
        expected.keys().collect::<Vec<_>>(),
        "the files written"
    );
    for (name, contents) in &preview {
        let page = text(contents);
        if name.ends_with(".html") {
            assert!(page.contains(NOINDEX), "{name}");
        }
        assert_eq!(indexed(&page), text(&expected[name]), "{name}");
    }
}

// Rule 212.
#[test]
fn a_preview_with_a_version_name_is_a_mistake_on_the_command_line() {
    let (written, run) = build(
        &[
            "--only",
            "changed.txt",
            "--preview",
            "--version-name",
            "pr-131",
        ],
        "",
        "preview-versioned",
    );
    assert_eq!(
        text(&run.stderr),
        "The option --preview builds a site without versions, so it cannot be given with --version-name.\n"
    );
    assert_eq!(run.status.code(), Some(2));
    assert!(!written.exists());
}
