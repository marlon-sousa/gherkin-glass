//! The output service over the real file system, checked against the versions fixture.

use std::fs::{create_dir_all, read_dir, read_to_string, write};
use std::path::PathBuf;
use std::sync::Arc;

use crate::adapters::OutputFolderStore;
use crate::entities::{FolderState, Marker, OutputProblemKind, VersionName};
use crate::fixtures::{self, copy_folder, scratch};
use crate::ports::OutputFolders;
use crate::services::OutputService;

fn fixture() -> PathBuf {
    fixtures::fixture("versions")
}

/// A fresh copy of the fixture's output folder, for a test that changes it.
fn output_copy(test: &str) -> PathBuf {
    let copied = scratch(&format!("output/{test}"));
    copy_folder(&fixture().join("output"), &copied);
    copied
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
    let output = scratch("output/new").join("site");
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
