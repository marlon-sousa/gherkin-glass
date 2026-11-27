//! What the output folder keeps, checked by running `ghg`.

#![expect(
    clippy::expect_used,
    reason = "a test file's helpers are test code, which clippy.toml lets panic"
)]

use std::fs::{create_dir_all, read_dir, read_to_string, remove_dir_all, write};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use rstest::rstest;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/keep-in-output")
}

/// An output folder of its own for `test`, holding what a GitHub Pages
/// branch holds beside a site: `.nojekyll`, `CNAME` and git's `.git` file.
fn host_folder(test: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("keep-in-output")
        .join(test);
    let _ = remove_dir_all(&folder);
    create_dir_all(&folder).expect("made");
    write(folder.join(".nojekyll"), "").expect("written");
    write(folder.join("CNAME"), "features.example.org\n").expect("written");
    write(folder.join(".git"), "gitdir: elsewhere\n").expect("written");
    folder
}

/// `ghg build` in `folder` of the fixture, writing into `output`, with
/// `arguments`: what it says on standard error, and its status.
fn ghg(folder: &str, output: &Path, arguments: &[&str]) -> (String, i32) {
    let result = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .arg("build")
        .arg("--output")
        .arg(output)
        .args(arguments)
        .current_dir(fixture().join(folder))
        .output()
        .expect("the command runs");
    assert_eq!(String::from_utf8(result.stdout).expect("UTF-8"), "");
    (
        String::from_utf8(result.stderr).expect("UTF-8"),
        result.status.code().expect("a status"),
    )
}

/// The configuration's patterns, given on the command line, which replaces them, before `more`.
fn host_patterns_and(more: &[&str]) -> Vec<String> {
    [".nojekyll", "/CNAME", ".git"]
        .iter()
        .chain(more)
        .map(|pattern| format!("--keep-in-output={pattern}"))
        .collect()
}

fn listing(folder: &Path) -> Vec<String> {
    let mut names: Vec<_> = read_dir(folder)
        .expect("listed")
        .map(|entry| {
            entry
                .expect("an entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}

/// The host's files are as `host_folder` wrote them.
fn assert_kept(output: &Path) {
    assert_eq!(read_to_string(output.join(".nojekyll")).expect("kept"), "");
    assert_eq!(
        read_to_string(output.join("CNAME")).expect("kept"),
        "features.example.org\n"
    );
    assert_eq!(
        read_to_string(output.join(".git")).expect("kept"),
        "gitdir: elsewhere\n"
    );
}

fn marker(output: &Path, name: &str) -> String {
    read_to_string(output.join(name)).expect("the marker is written")
}

// Rules 224, 225 and 227.
#[test]
fn a_folder_holding_only_what_is_kept_is_built_into_and_keeps_it() {
    let output = host_folder("site");
    assert_eq!(ghg(".", &output, &[]), (String::new(), 0));
    assert_kept(&output);
    assert!(output.join("billing/pay.html").is_file());
    let marker = marker(&output, "ghg-site.json");
    assert!(!marker.contains("CNAME") && !marker.contains(".nojekyll") && !marker.contains(".git"));
    // Rule 227: an earlier site is replaced, and what is kept stays.
    assert_eq!(ghg(".", &output, &[]), (String::new(), 0));
    assert_kept(&output);
}

// Rules 224 and 227: the same for versions.
#[test]
fn a_versioned_build_keeps_what_is_kept_beside_its_versions() {
    let output = host_folder("versions");
    assert_eq!(
        ghg(".", &output, &["--version-name", "v1.0"]),
        (String::new(), 0)
    );
    assert_eq!(
        ghg(".", &output, &["--version-name", "main"]),
        (String::new(), 0)
    );
    assert_eq!(
        ghg(".", &output, &["--version-name", "v1.0"]),
        (String::new(), 0)
    );
    assert_kept(&output);
    assert_eq!(
        listing(&output),
        [
            ".git",
            ".nojekyll",
            "CNAME",
            "assets",
            "data",
            "ghg-versions.json",
            "index.html",
            "main",
            "v1.0"
        ]
    );
}

// Rules 25 and 224: the command line's patterns win over the key's.
#[test]
fn patterns_on_the_command_line_replace_the_configuration_s() {
    let output = host_folder("command-line");
    let (report, status) = ghg(
        ".",
        &output,
        &["--keep-in-output", ".nojekyll", "--keep-in-output=.git"],
    );
    assert_eq!(status, 1);
    assert_eq!(
        report,
        format!(
            "{}: this is not a site ghg built, so it is left as it is. Build into an empty or new folder.\n",
            output.display()
        )
    );
    assert_kept(&output);
}

// Rule 228.
#[rstest]
#[case::a_folder_of_every_site(&[], "assets", "assets")]
#[case::every_page(&[], "billing/index.html", "*.html")]
#[case::a_folder_of_the_input(&[], "billing", "billing/")]
#[case::the_version_built(&["--version-name", "v1.0"], "v1.0", "v1.0")]
fn a_pattern_that_keeps_what_ghg_writes_stops_the_build_before_anything_is_written(
    #[case] arguments: &[&str],
    #[case] found: &str,
    #[case] pattern: &str,
) {
    let output = host_folder(&format!("clash-{pattern}").replace(['*', '/'], "_"));
    let mut arguments: Vec<String> = arguments
        .iter()
        .map(|argument| argument.to_string())
        .collect();
    arguments.extend(host_patterns_and(&[pattern]));
    let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let (report, status) = ghg(".", &output, &arguments);
    assert_eq!(status, 1);
    assert_eq!(
        report,
        format!(
            "{}/{found}: the pattern {pattern} keeps this, but ghg writes it or the folder it is in, so the output folder is left as it is.\n",
            output.display()
        )
    );
    assert_eq!(listing(&output), [".git", ".nojekyll", "CNAME"]);
}

// Rule 228: something kept in a folder ghg writes.
#[test]
fn something_kept_in_a_folder_ghg_wrote_stops_the_build() {
    let output = host_folder("in-assets");
    assert_eq!(ghg(".", &output, &[]), (String::new(), 0));
    write(output.join("assets/custom.css"), "mine").expect("written");
    let patterns = host_patterns_and(&["custom.css"]);
    let patterns: Vec<&str> = patterns.iter().map(String::as_str).collect();
    let (report, status) = ghg(".", &output, &patterns);
    assert_eq!(status, 1);
    assert_eq!(
        report,
        format!(
            "{}/assets/custom.css: the pattern custom.css keeps this, but ghg writes it or the folder it is in, so the output folder is left as it is.\n",
            output.display()
        )
    );
    assert_eq!(
        read_to_string(output.join("assets/custom.css")).expect("kept"),
        "mine"
    );
}

// Rule 226.
#[test]
fn a_mistaken_keep_in_output_is_reported_on_each_line() {
    let output = host_folder("mistaken");
    let (report, status) = ghg("mistaken", &output, &[]);
    assert_eq!(status, 1);
    assert_eq!(
        report,
        "ghg.toml:3: keep_in_output must not be empty.\n\
         ghg.toml:4: {draft is not a pattern in the gitignore syntax: unclosed alternate group; missing '}' (maybe escape '{' with '[{]'?)\n"
    );
}

// Rules 19 and 226.
#[rstest]
#[case::build("build")]
#[case::serve("serve")]
fn a_pattern_that_is_not_valid_is_a_mistake_on_the_command_line(#[case] command: &str) {
    let result = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .args([command, "--keep-in-output", "{draft"])
        .current_dir(fixture())
        .output()
        .expect("the command runs");
    assert_eq!(result.status.code(), Some(2));
    assert_eq!(
        String::from_utf8(result.stderr).expect("UTF-8"),
        "{draft is not a pattern in the gitignore syntax: unclosed alternate group; missing '}' (maybe escape '{' with '[{]'?)\n"
    );
}

// Rule 227: ghg serve builds into the output folder as ghg build does.
#[test]
fn ghg_serve_keeps_what_is_kept() {
    let output = host_folder("serve");
    let mut child = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .args(["serve", "--port", "0", "--output"])
        .arg(&output)
        .current_dir(fixture())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("ghg serve starts");
    let mut line = String::new();
    BufReader::new(child.stdout.take().expect("piped"))
        .read_line(&mut line)
        .expect("a line");
    let _ = child.kill();
    let _ = child.wait();
    assert!(line.starts_with("The site is served at "), "{line}");
    assert!(output.join("index.html").is_file());
    assert_kept(&output);
}
