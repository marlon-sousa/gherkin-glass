//! Versions, checked by running `ghg version add` and `ghg version remove` on the suites of
//! `tests/fixtures/versions`, and comparing the output folder with the ones committed beside them.

#![expect(
    clippy::expect_used,
    reason = "a test file's helpers are test code, which clippy.toml lets panic"
)]

use std::fs::{read_dir, read_to_string, remove_dir_all, write};
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "support/history_builds.rs"]
mod history_builds;
#[path = "support/site_files.rs"]
mod site_files;
#[path = "support/store_json.rs"]
mod store_json;
#[path = "support/version_placeholder.rs"]
mod version_placeholder;

use history_builds::{HISTORY_BUILDS, VERSION_BUILDS};
use site_files::{assert_same_site, assert_same_store, files};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/versions")
}

/// A fresh output folder for `test`, which does not exist yet.
fn output_folder(test: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("versions")
        .join(test);
    let _ = remove_dir_all(&folder);
    folder
}

/// `ghg` with `arguments` in the fixture's `folder`, writing into `output`: what it says on
/// standard error, and its status.
fn ghg(arguments: &[&str], folder: &str, output: &Path) -> (String, i32) {
    let result = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .args(arguments)
        .arg("--output")
        .arg(output)
        .current_dir(fixture().join(folder))
        .output()
        .expect("the command runs");
    assert_eq!(String::from_utf8(result.stdout).expect("UTF-8"), "");
    (
        String::from_utf8(result.stderr).expect("UTF-8"),
        result.status.code().expect("a status"),
    )
}

/// `ghg version add <version>` of the fixture's suite `suite`, into `output`, which succeeds.
fn add(suite: &str, version: &str, output: &Path) {
    let added = ghg(&["version", "add", version], suite, output);
    assert_eq!(added, (String::new(), 0), "{suite} as {version}");
}

/// A fresh output folder for `test`, holding the fixture's versions, added in turn.
fn with_versions(test: &str) -> PathBuf {
    let output = output_folder(test);
    for (suite, version) in VERSION_BUILDS {
        add(suite, version, &output);
    }
    output
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

// Rules 120 to 128, 253, 254 and 257: three versions added in turn, then one of them removed.
#[test]
fn versions_added_and_removed_leave_the_committed_output_folders() {
    let output = with_versions("added");
    assert_same_site(&output, &fixture().join("expected"));
    let removed = ghg(&["version", "remove", "v2.0.0-rc.1"], "suite", &output);
    assert_eq!(removed, (String::new(), 0));
    assert_same_site(&output, &fixture().join("expected-removed"));
}

// Rule 257: removing the last version removes the versions page and the lists of versions and
// pages, and removing one that is not there says so and changes nothing.
#[test]
fn the_last_version_removed_takes_the_versions_page_with_it() {
    let output = output_folder("last");
    add("suite", "v1.0.0", &output);
    assert_eq!(
        ghg(&["version", "remove", "v1.0.0"], "suite", &output),
        (String::new(), 0)
    );
    assert_eq!(listing(&output), ["ghg-site.json"]);
    let before = files(&output);
    assert_eq!(
        ghg(&["version", "remove", "v1.0.0"], "suite", &output),
        (
            format!(
                "{}: holds no version v1.0.0, so nothing was removed.\n",
                output.display()
            ),
            0
        )
    );
    assert_eq!(files(&output), before);
}

// Rule 121.
#[test]
fn a_name_that_cannot_be_a_version_s_is_a_mistake_on_the_command_line() {
    for name in ["release/1.0", "Previews", "data"] {
        let (report, status) = ghg(
            &["version", "add", name],
            "suite",
            &output_folder("bad-name"),
        );
        assert_eq!(status, 2, "{name}");
        assert!(
            report.starts_with(&format!(
                "{name} cannot be the name of a version or a preview. "
            )),
            "{report}"
        );
    }
}

// Rule 128: a version is listed among the others by its precedence.
#[test]
fn a_new_version_is_listed_with_the_others() {
    let output = with_versions("listed-with-others");
    add("suite", "v1.1.0", &output);
    let page = read_to_string(output.join("index.html")).expect("read");
    let first = page.find("v1.1.0/index.html").expect("listed");
    let second = page.find("v1.0.0/index.html").expect("listed");
    assert!(first < second, "{page}");
}

// Rule 123: a version whose input cannot be read leaves the earlier one as it was.
#[test]
fn a_version_that_fails_leaves_the_earlier_one_as_it_was() {
    let output = with_versions("fails");
    let before = files(&output);
    let (report, status) = ghg(&["version", "add", "v1.0.0"], "broken", &output);
    assert_eq!(status, 1);
    assert!(report.starts_with("broken.feature:4: "), "{report}");
    assert_eq!(files(&output), before);
}

// Rule 123: with replace_existing_version false, a version already there stops the run.
#[test]
fn a_version_kept_stops_the_run() {
    let output = with_versions("stops");
    let before = files(&output);
    let config = fixture().join("suite/keep.toml");
    let config = config.to_str().expect("UTF-8");
    let (report, status) = ghg(
        &["version", "add", "main", "--config", config],
        "suite",
        &output,
    );
    assert_eq!(status, 1);
    assert_eq!(
        report,
        format!(
            "{}/main: the version main is already built, and replace_existing_version is false.\n",
            output.display()
        )
    );
    assert_eq!(files(&output), before);
}

// Rule 124: a name that differs from one the store holds only in its capitals.
#[test]
fn a_version_named_as_another_but_for_its_capitals_stops_the_run() {
    let output = with_versions("capitals");
    let (report, status) = ghg(&["version", "add", "V1.0.0"], "suite", &output);
    assert_eq!(status, 1);
    assert_eq!(
        report,
        format!(
            "{}/V1.0.0: the output folder holds a version whose name differs from this one only in its capitals, which some systems would put in the same folder, so it is left as it is.\n",
            output.display()
        )
    );
}

// Rule 38: a marked folder must hold only what ghg wrote there, in a version's folder and
// beside the versions page alike.
#[test]
fn a_file_ghg_did_not_write_stops_the_run() {
    for (test, stray) in [
        ("in-a-version", "main/notes.txt"),
        ("in-assets", "assets/old.css"),
    ] {
        let output = with_versions(test);
        write(output.join(stray), "mine").expect("written");
        let before = files(&output);
        let (report, status) = ghg(&["version", "add", "v1.0.0"], "suite", &output);
        assert_eq!(status, 1, "{stray}");
        assert_eq!(
            report,
            format!(
                "{}/{stray}: ghg did not write this, so the output folder is left as it is.\n",
                output.display()
            )
        );
        assert_eq!(files(&output), before);
    }
}

// Rule 122: a version the store holds whose folder is gone stops the run.
#[test]
fn a_version_whose_folder_is_gone_stops_the_run() {
    let output = with_versions("gone");
    remove_dir_all(output.join("main")).expect("removed");
    let before = files(&output);
    let (report, status) = ghg(&["version", "add", "v1.0.0"], "suite", &output);
    assert_eq!(status, 1);
    assert_eq!(
        report,
        format!(
            "{}/main: the site's data holds this version, but its folder is missing, so the output folder is left as it is.\n",
            output.display()
        )
    );
    assert_eq!(files(&output), before);
}

// Rules 38 and 124: versions and a site without versions are never in one output folder.
#[test]
fn versions_and_a_site_without_versions_never_share_an_output_folder() {
    let output = output_folder("holds-site");
    assert_eq!(ghg(&["build"], "suite", &output), (String::new(), 0));
    let (report, status) = ghg(&["version", "add", "main"], "suite", &output);
    assert_eq!(status, 1);
    assert_eq!(
        report,
        format!(
            "{}: a site without versions is built here, so build it with ghg build, or add versions into another folder.\n",
            output.display()
        )
    );
    assert!(!output.join("main").exists());
    let output = with_versions("holds-versions");
    let (report, status) = ghg(&["build"], "suite", &output);
    assert_eq!(status, 1);
    assert_eq!(
        report,
        format!(
            "{}: the output folder holds versions of a site, so add a version with ghg version add, or build into another folder.\n",
            output.display()
        )
    );
}

// Rules 122, 126 and 211: a version added beside a preview leaves it as it was, and the
// versions page lists no preview.
#[test]
fn a_version_added_beside_a_preview_leaves_it_alone() {
    let output = output_folder("beside-a-preview");
    assert_eq!(
        ghg(&["preview", "add", "pr-1"], "suite", &output),
        (String::new(), 0)
    );
    let shell = read_to_string(output.join("previews/pr-1/index.html")).expect("read");
    let preview = read_to_string(output.join("data/previews/pr-1.json")).expect("read");
    add("next", "main", &output);
    assert_eq!(
        read_to_string(output.join("previews/pr-1/index.html")).expect("read"),
        shell
    );
    assert_eq!(
        read_to_string(output.join("data/previews/pr-1.json")).expect("read"),
        preview
    );
    let page = read_to_string(output.join("index.html")).expect("read");
    assert!(!page.contains("pr-1"), "{page}");
}

// The store of a history of versions: rules 247, 251, 253, 254 and 255.

// Rules 247 and 251: `moved` differs from `first` only by comments, blank lines and moved
// blocks, so they share their objects; `changed` and `draft` differ from it by one step each.
// Rule 255: `main` added again from `first` leaves `draft`'s object, which is removed.
#[test]
fn a_history_of_versions_shares_its_objects_and_removes_what_nothing_references() {
    let output = output_folder("history");
    let objects: Vec<usize> = HISTORY_BUILDS
        .iter()
        .map(|(suite, version)| {
            add(&format!("history/{suite}"), version, &output);
            listing(&output.join("data/objects")).len()
        })
        .collect();
    assert_eq!(objects, [2, 2, 3, 4, 3]);
    assert_same_store(&output, &fixture().join("history/expected"));
}
