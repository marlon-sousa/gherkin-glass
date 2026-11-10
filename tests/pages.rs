//! docs/BEHAVIOUR.md, "Pages", checked by building the fixtures and comparing
//! every file written with the site committed beside them.
//!
//! `cargo run -- build --config tests/fixtures/pages/ghg.toml`, and again with
//! `collapsed.toml`, writes the expected sites again after a change.

use std::collections::BTreeMap;
use std::fs::{create_dir_all, read, read_dir, remove_dir_all, write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// `ghg build` run in `folder` with `arguments` after it.
fn build(folder: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ghg"))
        .arg("build")
        .args(arguments)
        .current_dir(folder)
        .output()
        .expect("the command runs")
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

fn output(name: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = remove_dir_all(&folder);
    folder
}

fn assert_same_site(config: &str, expected: &str) {
    let written = output(expected);
    let result = build(
        &fixtures().join("pages"),
        &[
            "--config",
            config,
            "--output",
            written.to_str().expect("UTF-8"),
        ],
    );
    assert_eq!(String::from_utf8_lossy(&result.stderr), "");
    assert_eq!(result.status.code(), Some(0));
    let expected = files(&fixtures().join("pages").join(expected));
    let written = files(&written);
    assert_eq!(
        written.keys().collect::<Vec<_>>(),
        expected.keys().collect::<Vec<_>>(),
        "the files written"
    );
    for (name, contents) in &expected {
        assert!(
            written[name] == *contents,
            "{name} differs from the expected file:\n{}",
            String::from_utf8_lossy(&written[name])
        );
    }
}

// Rules 36 and 38 to 51: every page of the fixture, and the assets beside them.

#[test]
fn every_feature_file_has_its_page_as_the_expected_site_shows() {
    assert_same_site("ghg.toml", "expected");
}

#[test]
fn the_settings_start_rules_scenarios_and_backgrounds_collapsed() {
    assert_same_site("collapsed.toml", "expected-collapsed");
}

#[test]
fn the_assets_are_the_ones_built_into_ghg() {
    let expected = fixtures().join("pages/expected/assets");
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    assert_eq!(files(&expected), files(&repository));
}

#[test]
fn every_page_has_one_h1_and_a_lang_and_its_content_in_its_html() {
    for (name, page) in files(&fixtures().join("pages/expected")) {
        if !name.ends_with(".html") {
            continue;
        }
        let page = String::from_utf8(page).expect("UTF-8");
        assert_eq!(page.matches("<h1").count(), 1, "{name}");
        assert!(page.contains("<html lang=\""), "{name}");
        // Rule 32: no id the page gives its own parts may be an anchor; for
        // now every id is a block's.
        for id in page.split(" id=\"").skip(1) {
            let id = &id[..id.find('"').expect("closed")];
            assert!(!id.contains('_'), "{name}: {id}");
        }
    }
}

#[test]
fn the_page_s_lang_is_its_file_s_language_as_a_tag() {
    let site = fixtures().join("pages/expected");
    let lang = |page: &str| {
        let html = String::from_utf8(read(site.join(page)).expect("read")).expect("UTF-8");
        html.split("<html lang=\"").nth(1).expect("a lang")[..]
            .split('"')
            .next()
            .expect("closed")
            .to_string()
    };
    assert_eq!(lang("billing/refunds.html"), "en");
    assert_eq!(lang("saque em conta poupança.html"), "pt");
    assert_eq!(lang("emoji.html"), "und");
}

// Rule 37.

#[test]
fn two_files_with_the_same_page_stop_the_build_with_the_files_that_cannot_be_read() {
    let written = output("same-page");
    let result = build(
        &fixtures().join("same-page"),
        &["--input", ".", "--output", written.to_str().expect("UTF-8")],
    );
    // Names that differ only in case cannot share a folder in a checkout on
    // every system, so the service's own tests check them.
    assert_eq!(
        String::from_utf8_lossy(&result.stderr),
        "a.story: has the same page, a.html, as a.feature.\n\
         broken.feature:1: Unknown keyword: 'not a feature'.\n"
    );
    assert_eq!(result.status.code(), Some(1));
    assert!(!written.exists(), "nothing is written");
}

// Rule 38.

#[test]
fn the_output_folder_is_written_over_and_nothing_else_is_deleted() {
    let written = output("written-over");
    create_dir_all(written.join("billing")).expect("made");
    write(written.join("kept.txt"), "kept").expect("written");
    write(written.join("billing/refunds.html"), "old").expect("written");
    let arguments = ["--output", written.to_str().expect("UTF-8")];
    let result = build(&fixtures().join("pages"), &arguments);
    assert_eq!(result.status.code(), Some(0));
    assert_eq!(read(written.join("kept.txt")).expect("kept"), b"kept");
    let expected = read(fixtures().join("pages/expected/billing/refunds.html")).expect("read");
    assert_eq!(
        read(written.join("billing/refunds.html")).expect("read"),
        expected
    );
}

#[test]
fn a_file_that_cannot_be_written_stops_the_build() {
    // A file where the output folder should be cannot be written into.
    let written = output("not-a-folder");
    write(&written, "a file").expect("written");
    let given = written.to_str().expect("UTF-8");
    let result = build(&fixtures().join("pages"), &["--output", given]);
    let report = String::from_utf8_lossy(&result.stderr).into_owned();
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(report.lines().count(), 1, "{report}");
    assert!(
        report.starts_with(&format!(
            "{given}/billing/refunds.html: cannot be written: "
        )),
        "{report}"
    );
}
