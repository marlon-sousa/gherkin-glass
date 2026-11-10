//! docs/BEHAVIOUR.md, "Versions", checked on the file system and by running `ghg`.

use std::fs::{copy, create_dir_all, read_dir, read_to_string, remove_dir_all, remove_file, write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use gherkin_glass::{
    FolderState, Marker, OutputFolderStore, OutputFolders, OutputProblemKind, OutputService,
    VersionName,
};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/versions")
}

/// A fresh copy of the fixture's output folder, for a test that changes it.
fn output_copy(test: &str) -> PathBuf {
    let copied = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("versions")
        .join(test);
    let _ = remove_dir_all(&copied);
    copy_folder(&fixture().join("output"), &copied);
    copied
}

fn copy_folder(from: &Path, to: &Path) {
    create_dir_all(to).expect("the folder is made");
    for entry in read_dir(from).expect("the fixture is listed") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_folder(&entry.path(), &target);
        } else {
            copy(entry.path(), target).expect("the file is copied");
        }
    }
}

fn name(text: &str) -> VersionName {
    VersionName::new(text).expect("a version name")
}

fn names(list: &[VersionName]) -> Vec<&str> {
    list.iter().map(VersionName::as_str).collect()
}

fn service() -> OutputService {
    OutputService::new(Arc::new(OutputFolderStore))
}

// A versioned build: rules 120 to 122.

#[test]
fn a_version_name_that_cannot_be_one_is_a_mistake_on_the_command_line() {
    let output = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .args(["build", "--version-name", "release/1.0"])
        .current_dir(fixture().join("bad-name"))
        .output()
        .expect("the command runs");
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        String::from_utf8(output.stderr).expect("UTF-8"),
        "release/1.0 cannot be a version name. A version name is made of letters, digits, dots, hyphens, underscores and plus signs, starts with a letter or a digit, does not end with a dot, and is not assets, index.html, or a name Windows keeps for a device, such as con or nul.\n"
    );
}

#[test]
fn a_version_name_that_cannot_be_one_is_a_problem_of_the_configuration() {
    let output = Command::new(env!("CARGO_BIN_EXE_ghg"))
        .arg("build")
        .current_dir(fixture().join("bad-name"))
        .output()
        .expect("the command runs");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr).expect("UTF-8"),
        "ghg.toml:2: release/1.0 cannot be a version name. A version name is made of letters, digits, dots, hyphens, underscores and plus signs, starts with a letter or a digit, does not end with a dot, and is not assets, index.html, or a name Windows keeps for a device, such as con or nul.\n"
    );
}

#[test]
fn a_version_s_folder_is_cleared_and_marked() {
    let output = output_copy("cleared");
    let folder = service()
        .check(&output, &name("v1.0.0"), true)
        .expect("replaced");
    assert_eq!(folder.path(), output.join("v1.0.0"));
    service().make_room(&output, &folder).expect("made");
    let folder = folder.path();
    let left: Vec<_> = read_dir(folder)
        .expect("listed")
        .map(|entry| entry.expect("an entry").file_name())
        .collect();
    assert_eq!(left, ["ghg-version.json"]);
    assert_eq!(
        read_to_string(folder.join("ghg-version.json")).expect("read"),
        "{\"version\": \"v1.0.0\"}\n"
    );
    // The other versions and folders are left as they were.
    assert!(output.join("main/ghg-version.json").is_file());
    assert!(output.join("docs/notes.txt").is_file());
}

#[test]
fn a_new_version_s_folder_is_made_with_the_folders_above() {
    let output = Path::new(env!("CARGO_TARGET_TMPDIR")).join("versions/new/site");
    let _ = remove_dir_all(output.parent().expect("a parent"));
    let folder = service().check(&output, &name("main"), false).expect("new");
    service().make_room(&output, &folder).expect("made");
    assert_eq!(
        OutputFolderStore
            .state(folder.path(), &Marker::Version(name("main")))
            .expect("read"),
        FolderState::Marked
    );
}

// A version that is already there: rules 123 to 125.

#[test]
fn a_version_already_built_is_kept_when_replace_existing_version_is_false() {
    let problem = service()
        .check(&fixture().join("output"), &name("main"), false)
        .expect_err("kept");
    assert_eq!(problem.output, fixture().join("output"));
    assert_eq!(problem.version, Some(name("main")));
    assert_eq!(problem.kind, OutputProblemKind::AlreadyBuilt);
}

#[test]
fn a_folder_without_the_marker_is_never_replaced() {
    for replace in [true, false] {
        let problem = service()
            .check(&fixture().join("output"), &name("docs"), replace)
            .expect_err("left");
        assert_eq!(problem.kind, OutputProblemKind::NotAVersion);
    }
}

#[test]
fn a_file_in_the_version_s_place_is_never_replaced() {
    let output = output_copy("file");
    write(output.join("v3"), "a file").expect("written");
    let problem = service()
        .check(&output, &name("v3"), true)
        .expect_err("left");
    assert_eq!(problem.kind, OutputProblemKind::NotAVersion);
}

#[test]
fn an_empty_folder_is_used() {
    let output = output_copy("empty");
    create_dir_all(output.join("v3")).expect("made");
    let folder = service().check(&output, &name("v3"), false).expect("used");
    assert_eq!(folder.path(), output.join("v3"));
}

#[test]
fn a_folder_whose_name_differs_in_capitals_is_not_the_version() {
    // Where capitals are told apart, Main is simply missing; where they are
    // not, main takes its place.
    let output = fixture().join("output");
    match service().check(&output, &name("Main"), true) {
        Ok(folder) => assert!(!folder.path().exists()),
        Err(problem) => assert_eq!(problem.kind, OutputProblemKind::NotAVersion),
    }
}

#[test]
fn a_marker_that_is_a_folder_is_no_marker() {
    let output = output_copy("marker-folder");
    create_dir_all(output.join("v3/ghg-version.json")).expect("made");
    let problem = service()
        .check(&output, &name("v3"), true)
        .expect_err("left");
    assert_eq!(problem.kind, OutputProblemKind::NotAVersion);
    let list = service().versions(&output).expect("listed");
    assert!(!names(&list.releases).contains(&"v3"));
}

#[cfg(unix)]
mod links {
    use std::os::unix::fs::symlink;

    use super::*;

    #[test]
    fn a_link_in_the_version_s_place_is_never_followed() {
        let output = output_copy("link-place");
        symlink(output.join("main"), output.join("v3")).expect("linked");
        let problem = service()
            .check(&output, &name("v3"), true)
            .expect_err("left");
        assert_eq!(problem.kind, OutputProblemKind::NotAVersion);
        // A linked folder with a marker is not listed either.
        let list = service().versions(&output).expect("listed");
        assert_eq!(names(&list.development), ["main"]);
    }

    #[test]
    fn a_link_inside_a_version_goes_without_what_it_points_to() {
        let output = output_copy("link-inside");
        symlink(output.join("docs"), output.join("v1.0.0/docs")).expect("linked");
        symlink(
            output.join("docs/notes.txt"),
            output.join("v1.0.0/notes.txt"),
        )
        .expect("linked");
        let folder = service()
            .check(&output, &name("v1.0.0"), true)
            .expect("replaced");
        service().make_room(&output, &folder).expect("made");
        assert!(!output.join("v1.0.0/docs").exists());
        assert!(!output.join("v1.0.0/notes.txt").exists());
        assert!(output.join("docs/notes.txt").is_file());
    }
}

// The versions page: rules 126 to 128.

#[test]
fn only_marked_folders_are_versions_in_the_page_s_order() {
    let list = service()
        .versions(&fixture().join("output"))
        .expect("listed");
    assert_eq!(names(&list.development), ["main"]);
    assert_eq!(names(&list.releases), ["v1.0.0"]);
    assert_eq!(names(&list.pre_releases), ["v2.0.0-rc.1"]);
}

#[test]
fn an_output_folder_not_made_yet_holds_no_versions() {
    let list = service()
        .versions(&fixture().join("no-such-output"))
        .expect("listed");
    assert!(list.development.is_empty() && list.releases.is_empty());
}

// A whole versioned build, run with `ghg`.

/// `ghg build` in the fixture's `folder`, writing into `output`, with `arguments`.
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

#[test]
fn a_versioned_build_replaces_its_version_and_writes_the_versions_page() {
    let output = output_copy("build");
    let (report, status) = ghg("suite", &output, &["--version-name", "v1.0.0"]);
    assert_eq!((report.as_str(), status), ("", 0));
    assert_eq!(
        listing(&output.join("v1.0.0")),
        [
            "assets",
            "ghg-version.json",
            "index.html",
            "steps.html",
            "tags",
            "withdraw.html"
        ]
    );
    assert_eq!(
        listing(&output),
        [
            "assets",
            "docs",
            "index.html",
            "main",
            "v1.0.0",
            "v2.0.0-rc.1"
        ]
    );
    assert!(output.join("docs/notes.txt").is_file());
    assert!(output.join("assets/gherkin-glass.css").is_file());
    let page = read_to_string(output.join("index.html")).expect("read");
    let expected = read_to_string(fixture().join("expected/index.html")).expect("read");
    assert_eq!(page, expected);
}

#[test]
fn a_new_version_is_listed_with_the_others() {
    let output = output_copy("listed-with-others");
    let (report, status) = ghg("suite", &output, &["--version-name", "v1.1.0"]);
    assert_eq!((report.as_str(), status), ("", 0));
    let page = read_to_string(output.join("index.html")).expect("read");
    let first = page.find("v1.1.0/index.html").expect("listed");
    let second = page.find("v1.0.0/index.html").expect("listed");
    assert!(first < second, "{page}");
}

#[test]
fn a_build_that_fails_leaves_the_earlier_version_as_it_was() {
    let output = output_copy("fails");
    let (report, status) = ghg("broken", &output, &["--version-name", "v1.0.0"]);
    assert_eq!(status, 1);
    assert!(report.starts_with("broken.feature:4: "), "{report}");
    assert!(output.join("v1.0.0/sub/old.html").is_file());
    assert_eq!(
        read_to_string(output.join("index.html")).expect("read"),
        "<!doctype html>\n"
    );
}

#[test]
fn a_version_kept_or_a_folder_that_is_not_one_stops_the_build() {
    let output = output_copy("stops");
    let shown = output.display();
    let config = fixture().join("suite/keep.toml");
    let config = config.to_str().expect("UTF-8");
    let (report, status) = ghg(
        "suite",
        &output,
        &["--version-name", "main", "--config", config],
    );
    assert_eq!(status, 1);
    assert_eq!(
        report,
        format!(
            "{shown}/main: the version main is already built, and replace_existing_version is false.\n"
        )
    );
    let (report, status) = ghg("suite", &output, &["--version-name", "docs"]);
    assert_eq!(status, 1);
    assert_eq!(
        report,
        format!("{shown}/docs: this is not a version ghg built, so it is left as it is.\n")
    );
    assert_eq!(listing(&output.join("main")), ["ghg-version.json"]);
    assert_eq!(listing(&output.join("docs")), ["notes.txt"]);
    assert_eq!(
        read_to_string(output.join("index.html")).expect("read"),
        "<!doctype html>\n"
    );
}

#[test]
fn what_is_at_the_page_s_place_and_its_assets_is_replaced() {
    let output = output_copy("root");
    remove_file(output.join("index.html")).expect("removed");
    create_dir_all(output.join("index.html/inside")).expect("made");
    create_dir_all(output.join("assets")).expect("made");
    write(output.join("assets/old.css"), "old").expect("written");
    let (report, status) = ghg("suite", &output, &["--version-name", "main"]);
    assert_eq!((report.as_str(), status), ("", 0));
    assert!(output.join("index.html").is_file());
    assert_eq!(
        listing(&output.join("assets")),
        ["gherkin-glass.css", "gherkin-glass.js"]
    );
}

#[test]
fn an_output_folder_holding_a_site_without_versions_stops_a_versioned_build() {
    let output = Path::new(env!("CARGO_TARGET_TMPDIR")).join("versions/holds-site");
    let _ = remove_dir_all(&output);
    let (report, status) = ghg("suite", &output, &[]);
    assert_eq!((report.as_str(), status), ("", 0));
    let (report, status) = ghg("suite", &output, &["--version-name", "main"]);
    assert_eq!(status, 1);
    assert_eq!(
        report,
        format!(
            "{}: a site without versions is built here, so build without a version name, or into another folder.\n",
            output.display()
        )
    );
    assert!(!output.join("main").exists());
}
