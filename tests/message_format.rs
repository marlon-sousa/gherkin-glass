//! What `ghg build --message-format json` prints on standard output, checked by running it on
//! fixtures that build, that have nothing to build and that stop the build.

#![expect(
    clippy::expect_used,
    reason = "a test file's helpers are test code, which clippy.toml lets panic"
)]

use std::fs::remove_dir_all;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// `ghg build --message-format json` run in `folder`, with `arguments`, writing into a fresh
/// folder of its own named `name`, which it gives back with the run's output.
fn build(folder: &Path, arguments: &[&str], name: &str) -> (PathBuf, Output) {
    let output = output_folder(name);
    let _ = remove_dir_all(&output);
    (output.clone(), run(&["build"], folder, &output, arguments))
}

fn output_folder(name: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("message-format")
        .join(name)
}

/// `ghg`, `command` and `--message-format json` run in `folder`, writing into `output`, with
/// `arguments`.
fn run(command: &[&str], folder: &Path, output: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ghg"))
        .args(command)
        .args(["--message-format", "json", "--output"])
        .arg(output)
        .args(arguments)
        .current_dir(folder)
        .output()
        .expect("the command runs")
}

/// The one JSON object of standard output, which is one line.
fn report(run: &Output) -> Value {
    let stdout = String::from_utf8(run.stdout.clone()).expect("UTF-8");
    assert_eq!(stdout.lines().count(), 1, "{stdout}");
    serde_json::from_str(&stdout).expect("JSON")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

// Rule 231: a site written, with its counts and its notes, each note also said on standard
// error as without the option.
#[test]
fn a_site_written_is_reported_with_its_counts_and_notes() {
    let (output, run) = build(&fixture("constructs"), &["--input", "."], "built");
    assert_eq!(run.status.code(), Some(0));
    let no_title = "The site has no title, so its home page is called Feature files. Set title in ghg.toml, or give --title.";
    assert_eq!(text(&run.stderr), format!("{no_title}\n"));
    let reported = report(&run);
    assert_eq!(reported["outcome"], "built");
    assert_eq!(reported["output"], output.to_string_lossy().as_ref());
    assert_eq!(reported["version"], Value::Null);
    assert_eq!(reported["preview"], Value::Null);
    assert_eq!(reported["features"], 2);
    assert_eq!(reported["scenarios"], 4);
    assert_eq!(
        reported["notes"],
        json!([{ "kind": "no-title", "message": no_title }])
    );
    assert_eq!(reported["errors"], json!([]));
}

// Rules 229 and 231.
#[test]
fn nothing_to_build_is_reported_with_why() {
    let (output, run) = build(&fixture("no-features"), &[], "nothing");
    assert_eq!(run.status.code(), Some(0));
    let why = "features: the input folder holds no feature file, so no site was built.";
    assert_eq!(
        report(&run),
        json!({
            "outcome": "nothing-to-build",
            "output": null,
            "version": null,
            "preview": null,
            "features": null,
            "scenarios": null,
            "notes": [{ "kind": "nothing-to-build", "message": why }],
            "errors": [],
        })
    );
    assert!(!output.exists());
}

// Rules 16 and 231: each problem of the report, with its path, its line and its message.
#[test]
fn a_stopped_build_is_reported_with_every_problem() {
    let (_, run) = build(&fixture("unreadable"), &["--input", "."], "stopped");
    assert_eq!(run.status.code(), Some(1));
    let reported = report(&run);
    assert_eq!(reported["outcome"], "stopped");
    assert_eq!(
        reported["errors"][0],
        json!({ "path": "empty.feature", "line": 1, "message": "Syntax error." })
    );
    let errors = reported["errors"].as_array().expect("a list");
    assert_eq!(errors.len(), text(&run.stderr).lines().count());
}

// Rule 19: a format ghg does not have is a mistake on the command line, which prints no JSON.
#[test]
fn a_format_ghg_does_not_have_is_a_mistake() {
    let (_, run) = build(
        &fixture("no-features"),
        &["--message-format", "xml"],
        "mistaken",
    );
    assert_eq!(run.status.code(), Some(2));
    assert_eq!(text(&run.stdout), "");
}

// Rules 120, 211 and 231: a version's name, or a preview's, is reported with
// what was written as it.
#[test]
fn a_version_or_a_preview_added_is_reported_with_its_name() {
    let output = output_folder("added");
    let _ = remove_dir_all(&output);
    let input = ["--input", "."];
    let added = run(
        &["version", "add", "v1"],
        &fixture("constructs"),
        &output,
        &input,
    );
    assert_eq!(added.status.code(), Some(0));
    let reported = report(&added);
    assert_eq!(reported["outcome"], "built");
    assert_eq!(reported["version"], "v1");
    assert_eq!(reported["preview"], Value::Null);
    assert_eq!(reported["features"], 2);
    let added = run(
        &["preview", "add", "pr-1"],
        &fixture("constructs"),
        &output,
        &input,
    );
    assert_eq!(added.status.code(), Some(0));
    let reported = report(&added);
    assert_eq!(reported["outcome"], "built");
    assert_eq!(reported["version"], Value::Null);
    assert_eq!(reported["preview"], "pr-1");
}

// Rules 231, 257 and 258: a version or a preview removed, and one there was
// none of.
#[test]
fn a_removal_is_reported_and_so_is_nothing_to_remove() {
    let output = output_folder("removed");
    let _ = remove_dir_all(&output);
    let constructs = fixture("constructs");
    let added = run(
        &["version", "add", "v1"],
        &constructs,
        &output,
        &["--input", "."],
    );
    assert_eq!(added.status.code(), Some(0));
    let removed = run(&["version", "remove", "v1"], &constructs, &output, &[]);
    assert_eq!(removed.status.code(), Some(0));
    let reported = report(&removed);
    assert_eq!(reported["outcome"], "removed");
    assert_eq!(reported["version"], "v1");
    assert_eq!(reported["features"], Value::Null);
    let none = run(&["preview", "remove", "pr-1"], &constructs, &output, &[]);
    assert_eq!(none.status.code(), Some(0));
    let why = format!(
        "{}: holds no preview pr-1, so nothing was removed.",
        output.display()
    );
    assert_eq!(text(&none.stderr), format!("{why}\n"));
    assert_eq!(
        report(&none),
        json!({
            "outcome": "nothing-to-remove",
            "output": null,
            "version": null,
            "preview": null,
            "features": null,
            "scenarios": null,
            "notes": [{ "kind": "nothing-to-remove", "message": why }],
            "errors": [],
        })
    );
}

// Rules 16 and 231: a problem with no line, such as a configuration file that cannot be read,
// is reported with a line of null.
#[test]
fn a_problem_with_no_line_is_reported_with_a_line_of_null() {
    let (_, run) = build(
        &fixture("no-features"),
        &["--config", "missing.toml"],
        "no-line",
    );
    assert_eq!(run.status.code(), Some(1));
    let reported = report(&run);
    assert_eq!(reported["outcome"], "stopped");
    let error = &reported["errors"][0];
    assert_eq!(error["path"], "missing.toml");
    assert_eq!(error["line"], Value::Null);
    assert!(
        error["message"]
            .as_str()
            .is_some_and(|message| message.starts_with("cannot be read: ")),
        "{error}"
    );
}
