//! The site writer's checks over the real file system, against the versions fixture.

use std::fs::{create_dir_all, remove_dir_all, remove_file, write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rstest::rstest;

use crate::adapters::{FolderFiles, MemoryFiles, OutputFolderStore};
use crate::entities::{BuildTarget, Record, VersionName};
use crate::errors::{OutputError, OutputErrorKind};
use crate::fixtures::{self, copy_folder, scratch};
use crate::policies::Moment;
use crate::services::SiteWriter;

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

fn version(text: &str) -> BuildTarget {
    BuildTarget::Version(name(text))
}

const REPLACE: Moment = Moment::Checking { replace: true };
const KEEP: Moment = Moment::Checking { replace: false };

fn check(
    output: &Path,
    target: &BuildTarget,
    moment: Moment,
) -> Result<Option<Record>, OutputError> {
    let writer = SiteWriter::new(
        Arc::new(OutputFolderStore),
        Arc::new(FolderFiles),
        Arc::new(MemoryFiles::default()),
    );
    writer
        .check_folder(output, target, moment)
        .map(|folder| folder.record().cloned())
}

fn refused(output: &Path, target: &BuildTarget) -> OutputErrorKind {
    check(output, target, REPLACE).expect_err("refused").kind
}

#[test]
fn a_folder_holding_only_what_ghg_wrote_is_safe_and_its_versions_read() {
    let record = check(&fixture().join("output"), &version("v1.0.0"), REPLACE).expect("safe");
    let Some(Record::Versions { page, versions }) = record else {
        panic!("{record:?}");
    };
    assert_eq!(page.into_iter().collect::<Vec<_>>(), ["index.html"]);
    let names: Vec<_> = versions.keys().map(VersionName::as_str).collect();
    assert_eq!(names, ["main", "v1.0.0", "v2.0.0-rc.1"]);
}

#[test]
fn a_version_already_built_is_kept_when_replace_existing_version_is_false() {
    let output = fixture().join("output");
    let error = check(&output, &version("main"), KEEP).expect_err("kept");
    assert_eq!(error.version, Some(name("main")));
    assert_eq!(error.kind, OutputErrorKind::AlreadyBuilt);
    assert!(check(&output, &version("v3"), KEEP).is_ok());
}

#[rstest]
#[case::with_a_version(version("v1.0.0"))]
#[case::with_a_new_version(version("v3"))]
fn something_ghg_did_not_write_stops_the_build_at_its_outermost_folder(
    #[case] target: BuildTarget,
) {
    let kind = refused(&fixture().join("unsafe"), &target);
    assert_eq!(
        kind,
        OutputErrorKind::NotSafe {
            found: "docs".into()
        }
    );
}

#[test]
fn a_file_added_inside_a_version_stops_the_build() {
    let output = output_copy("added");
    write(output.join("v1.0.0/sub/extra.html"), "extra").expect("written");
    let kind = refused(&output, &version("main"));
    assert_eq!(
        kind,
        OutputErrorKind::NotSafe {
            found: "v1.0.0/sub/extra.html".into()
        }
    );
}

#[test]
fn a_version_the_marker_lists_whose_folder_is_missing_stops_the_build() {
    let output = output_copy("missing");
    remove_dir_all(output.join("main")).expect("removed");
    let kind = refused(&output, &version("v3"));
    assert_eq!(kind, OutputErrorKind::MissingVersion { name: name("main") });
}

#[test]
fn a_file_ghg_wrote_and_that_is_gone_is_no_stray() {
    let output = output_copy("gone");
    remove_file(output.join("v1.0.0/sub/old.html")).expect("removed");
    assert!(check(&output, &version("v3"), REPLACE).is_ok());
}

#[test]
fn a_name_that_differs_from_a_version_s_only_in_its_capitals_is_refused() {
    let kind = refused(&fixture().join("output"), &version("Main"));
    assert_eq!(kind, OutputErrorKind::OtherCapitals);
}

#[rstest]
#[case::missing(None)]
#[case::empty(Some(""))]
fn a_missing_or_empty_folder_is_used_with_nothing_recorded(#[case] made: Option<&str>) {
    let output = scratch(&format!("output/unmade-{}", made.is_some())).join("site");
    if made.is_some() {
        create_dir_all(&output).expect("made");
    }
    assert_eq!(
        check(&output, &BuildTarget::Site, KEEP).expect("used"),
        None
    );
}

#[test]
fn a_marker_that_is_a_folder_is_no_marker() {
    let output = output_copy("marker-folder");
    remove_file(output.join(Record::VERSIONS_MARKER)).expect("removed");
    create_dir_all(output.join(Record::VERSIONS_MARKER)).expect("made");
    assert_eq!(refused(&output, &version("v3")), OutputErrorKind::NotASite);
}

#[rstest]
#[case::a_site_into_versions(BuildTarget::Site, OutputErrorKind::HoldsVersions)]
#[case::a_version_into_versions(version("v3"), OutputErrorKind::NotSafe { found: "ghg-site.json".into() })]
fn a_folder_holding_both_markers_is_read_as_versions(
    #[case] target: BuildTarget,
    #[case] kind: OutputErrorKind,
) {
    let output = output_copy(&format!("both-{}", matches!(target, BuildTarget::Site)));
    write(output.join(Record::SITE_MARKER), "{\"files\": []}").expect("written");
    assert_eq!(refused(&output, &target), kind);
}

#[cfg(unix)]
mod links {
    use std::os::unix::fs::symlink;

    use super::*;
    use crate::ports::OutputFolders;

    #[test]
    fn a_link_is_never_followed_and_stops_the_build() {
        let output = output_copy("link");
        symlink(output.join("main"), output.join("v1.0.0/linked")).expect("linked");
        let kind = refused(&output, &version("v1.0.0"));
        assert_eq!(
            kind,
            OutputErrorKind::NotSafe {
                found: "v1.0.0/linked".into()
            }
        );
    }

    #[test]
    fn clearing_a_folder_removes_a_link_and_never_what_it_points_to() {
        let output = output_copy("link-cleared");
        let kept = scratch("output/link-kept");
        write(kept.join("notes.txt"), "kept").expect("written");
        symlink(&kept, output.join("v1.0.0/kept")).expect("linked");
        OutputFolderStore.clear(&output).expect("cleared");
        assert!(kept.join("notes.txt").is_file());
    }
}
