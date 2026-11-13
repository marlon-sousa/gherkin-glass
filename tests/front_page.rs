//! The front page, checked by building `tests/fixtures/front-page` and comparing what is
//! written with what is committed beside it, which tests/fixtures/README.md describes.

#![expect(
    clippy::expect_used,
    reason = "a test file's helpers are test code, which clippy.toml lets panic"
)]

use std::collections::BTreeMap;
use std::fs::{create_dir_all, read, read_dir, read_to_string, remove_dir_all, write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/front-page")
}

/// `ghg build` run in `folder` with `arguments`, writing into a fresh folder named `name`,
/// which it returns with the run's output.
fn build(folder: &Path, arguments: &[&str], name: &str) -> (PathBuf, Output) {
    let output = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("front-page")
        .join(name);
    let _ = remove_dir_all(&output);
    let run = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .arg("build")
        .args(arguments)
        .arg("--output")
        .arg(&output)
        .current_dir(folder)
        .output()
        .expect("the command runs");
    (output, run)
}

/// Every file below `folder`, by its `/`-joined path in it.
fn files(folder: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut found = BTreeMap::new();
    let mut folders = vec![(folder.to_path_buf(), String::new())];
    while let Some((path, prefix)) = folders.pop() {
        for entry in read_dir(&path).expect("the folder is listed") {
            let entry = entry.expect("the folder is listed");
            let name = format!("{prefix}{}", entry.file_name().to_string_lossy());
            if entry.path().is_dir() {
                folders.push((entry.path(), format!("{name}/")));
            } else {
                found.insert(name, read(entry.path()).expect("the file is read"));
            }
        }
    }
    found
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn the_home_page_of_a_site_without_versions_is_written_from_the_template() {
    // Rules 183, 185 and 187 to 190: the template's heading, text, table and link around
    // ghg's part, whose headings are one level below the last heading before the marker.
    let (written, run) = build(&fixture(), &["--config", "ghg.toml"], "site");
    assert_eq!(text(&run.stderr), "");
    assert_eq!(run.status.code(), Some(0));
    let expected = files(&fixture().join("expected"));
    let written = files(&written);
    assert_eq!(
        written.keys().collect::<Vec<_>>(),
        expected.keys().collect::<Vec<_>>(),
        "the files written"
    );
    for (name, contents) in &expected {
        assert!(
            written.get(name) == Some(contents),
            "{name} differs from the expected file:\n{}",
            written
                .get(name)
                .map(|bytes| text(bytes))
                .unwrap_or_default()
        );
    }
}

#[test]
fn the_versions_page_of_a_versioned_build_is_written_from_the_template() {
    // Rule 183: the versions page takes the template, and the version's own home
    // page does not.
    let (written, run) = build(&fixture(), &["--config", "versioned.toml"], "versions");
    assert_eq!(text(&run.stderr), "");
    assert_eq!(run.status.code(), Some(0));
    let page = |path: &Path| read_to_string(path).expect("the page is read");
    assert_eq!(
        page(&written.join("index.html")),
        page(&fixture().join("expected-versions.html"))
    );
    let home = page(&written.join("v1.0.0/index.html"));
    assert!(home.contains("<h1>Banking</h1>"), "{home}");
    assert!(!home.contains("The Banking suite"), "{home}");
}

#[test]
fn the_option_wins_over_the_key_and_a_template_that_breaks_a_rule_stops_the_build() {
    // Rules 183 and 190: `--front-page` is relative to the current folder, and raw HTML stops
    // the build, naming the template's line, with nothing written.
    let (written, run) = build(
        &fixture(),
        &["--config", "ghg.toml", "--front-page", "broken/raw-html.md"],
        "raw-html",
    );
    assert_eq!(
        text(&run.stderr),
        "broken/raw-html.md:5: raw HTML cannot be written in a front page; only the marker <!-- ghg:contents --> and other comments can.\n"
    );
    assert_eq!(run.status.code(), Some(1));
    assert!(!written.exists());
}

#[test]
fn a_template_that_cannot_be_read_stops_the_build() {
    // Rule 186: the key's path is relative to the configuration file's folder.
    let (written, run) = build(&fixture(), &["--config", "broken/missing.toml"], "missing");
    let report = text(&run.stderr);
    assert!(
        report.starts_with(&format!(
            "{}: cannot be read: ",
            Path::new("broken").join("nowhere.md").display()
        )),
        "{report}"
    );
    assert_eq!(run.status.code(), Some(1));
    assert!(!written.exists());
}

#[test]
fn a_template_with_raw_html_named_by_the_key_is_reported_as_the_settings_give_it() {
    // Rule 190.
    let (_, run) = build(
        &fixture().join("broken"),
        &["--config", "raw-html.toml"],
        "raw-html-key",
    );
    assert_eq!(
        text(&run.stderr),
        "raw-html.md:5: raw HTML cannot be written in a front page; only the marker <!-- ghg:contents --> and other comments can.\n"
    );
    assert_eq!(run.status.code(), Some(1));
}

#[test]
fn a_feature_file_that_cannot_be_read_is_reported_before_the_template() {
    // Rule 186: the template is read only once the feature files are read and parsed.
    let input = Path::new(env!("CARGO_TARGET_TMPDIR")).join("front-page-input");
    let _ = remove_dir_all(&input);
    create_dir_all(&input).expect("the input is made");
    write(input.join("broken.feature"), "").expect("written");
    let input = input.to_str().expect("UTF-8");
    let (_, run) = build(
        &fixture(),
        &["--input", input, "--front-page", "nowhere.md"],
        "feature-first",
    );
    let report = text(&run.stderr);
    assert!(report.starts_with("broken.feature:"), "{report}");
    assert!(!report.contains("nowhere.md"), "{report}");
    assert_eq!(run.status.code(), Some(1));
}
