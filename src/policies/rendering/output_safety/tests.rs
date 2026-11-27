use std::collections::{BTreeMap, BTreeSet};

use rstest::rstest;

use super::*;
use crate::test_support::{folder, kept_in_output, marker, name, set};

const KEEP: Moment<'static> = Moment::Checking { replace: false };
const REPLACE: Moment<'static> = Moment::Checking { replace: true };
const WRITING: Moment<'static> = Moment::Writing {
    site: &BTreeSet::new(),
};

fn version(text: &str) -> BuildTarget {
    BuildTarget::Version(name(text))
}

fn preview(text: &str) -> BuildTarget {
    BuildTarget::Preview(name(text))
}

/// What the policy says of adding `target` to `contents`, with nothing kept.
fn judged(
    contents: OutputContents,
    target: &BuildTarget,
    moment: Moment<'_>,
) -> Result<SafeFolder, OutputErrorKind> {
    output_safety(
        contents,
        target,
        Change::Add,
        moment,
        &KeptInOutput::default(),
    )
}

/// The folders that hold `files`, each named once.
fn folders_of_files(files: &[&str]) -> Vec<String> {
    let folders: BTreeSet<String> = files.iter().flat_map(|file| folders_of(file)).collect();
    folders.into_iter().collect()
}

/// A folder ghg marked, holding the marker and `files`, every one of them listed.
fn marked(files: &[&str]) -> OutputContents {
    let folders = folders_of_files(files);
    let folders: Vec<&str> = folders.iter().map(String::as_str).collect();
    let found: Vec<&str> = files.iter().copied().chain(["ghg-site.json"]).collect();
    folder(&found, &folders, Some(marker(files)))
}

/// The store of a site without versions.
const SITE: [&str; 3] = ["index.html", "data/site.json", "data/objects/a.json"];

/// The store of the versions `main` and `v1.0.0`.
const VERSIONS: [&str; 6] = [
    "index.html",
    "main/index.html",
    "v1.0.0/index.html",
    "data/versions/main.json",
    "data/versions/v1.0.0.json",
    "data/versions.json",
];

/// The store of the preview `pr-1`.
const PREVIEW: [&str; 2] = ["previews/pr-1/index.html", "data/previews/pr-1.json"];

#[rstest]
#[case::missing(OutputContents::Missing)]
#[case::empty(folder(&[], &[], None))]
fn a_missing_or_empty_folder_is_safe_and_lists_nothing(#[case] contents: OutputContents) {
    let safe = judged(contents, &BuildTarget::Site, KEEP);
    assert_eq!(safe, Ok(SafeFolder::default()));
}

// Rule 38.
#[rstest]
#[case::a_file(OutputContents::NotAFolder)]
#[case::a_folder_with_no_marker(folder(&["notes.txt"], &[], None))]
#[case::a_marker_that_is_not_one(folder(&["ghg-site.json"], &[], Some("[]".into())))]
#[case::a_marker_listing_a_path_outside(folder(
    &["ghg-site.json"],
    &[],
    Some(r#"{"format":1,"ghg":"2.0.0","files":["../notes.txt"]}"#.into())
))]
fn a_folder_ghg_did_not_mark_is_not_a_site(#[case] contents: OutputContents) {
    let judged = judged(contents, &BuildTarget::Site, REPLACE);
    assert_eq!(judged, Err(OutputErrorKind::NotASite));
}

// Rule 256.
#[rstest]
#[case::its_versions_marker(folder(&["ghg-versions.json", "index.html"], &[], None))]
#[case::its_site_marker(folder(
    &["ghg-site.json", "index.html"],
    &[],
    Some(r#"{"files":["index.html"]}"#.into())
))]
fn a_folder_of_gherkin_glass_one_is_refused_first(#[case] contents: OutputContents) {
    let judged = judged(contents, &version("v1"), REPLACE);
    assert_eq!(judged, Err(OutputErrorKind::GherkinGlassOne));
}

// Rule 246.
#[test]
fn a_store_newer_than_this_ghg_s_is_refused_naming_its_ghg() {
    let newer = folder(
        &["ghg-site.json", "index.html"],
        &[],
        Some(r#"{"format":2,"ghg":"3.0.0","files":["index.html"]}"#.into()),
    );
    let judged = judged(newer, &BuildTarget::Site, REPLACE);
    assert_eq!(
        judged,
        Err(OutputErrorKind::NewerStore {
            ghg: "3.0.0".into()
        })
    );
}

// Rule 38: the first stray thing, folder by folder, a folder named rather than what it holds.
#[test]
fn a_marked_folder_holding_what_ghg_did_not_write_is_not_safe() {
    let OutputContents::Folder {
        mut files,
        mut folders,
        kept,
        marker,
    } = marked(&SITE)
    else {
        panic!("a folder");
    };
    files.extend(set(&["z.txt", "notes/a.txt", "data/b.txt"]));
    folders.insert("notes".into());
    let contents = OutputContents::Folder {
        files,
        folders,
        kept,
        marker,
    };
    let judged = judged(contents, &BuildTarget::Site, REPLACE);
    assert_eq!(
        judged,
        Err(OutputErrorKind::NotSafe {
            found: "data/b.txt".into()
        })
    );
}

#[test]
fn a_safe_folder_gives_what_its_marker_lists_and_what_it_holds() {
    let listed: Vec<&str> = SITE.iter().copied().chain(["gone.json"]).collect();
    let contents = folder(
        &[
            "ghg-site.json",
            "index.html",
            "data/site.json",
            "data/objects/a.json",
        ],
        &["data", "data/objects"],
        Some(marker(&listed)),
    );
    let safe = judged(contents, &BuildTarget::Site, REPLACE).expect("safe");
    assert_eq!(safe.listed, set(&listed));
    assert_eq!(safe.found, set(&SITE));
}

// Rules 38 and 124: what a folder's store holds decides which builds it takes.
#[rstest]
#[case::a_site_beside_versions(&VERSIONS, BuildTarget::Site, Err(OutputErrorKind::HoldsVersions))]
#[case::a_version_beside_a_site(&SITE, version("v2"), Err(OutputErrorKind::HoldsSite))]
#[case::a_site_beside_previews(&PREVIEW, BuildTarget::Site, Ok(()))]
#[case::a_version_beside_previews(&PREVIEW, version("v2"), Ok(()))]
#[case::a_preview_beside_a_site(&SITE, preview("pr-2"), Ok(()))]
#[case::a_preview_beside_versions(&VERSIONS, preview("pr-2"), Ok(()))]
#[case::a_version_of_other_capitals(&VERSIONS, version("Main"), Err(OutputErrorKind::OtherCapitals))]
#[case::a_preview_of_other_capitals(&PREVIEW, preview("PR-1"), Err(OutputErrorKind::OtherCapitals))]
fn a_store_takes_only_builds_beside_what_it_holds(
    #[case] files: &[&str],
    #[case] target: BuildTarget,
    #[case] outcome: Result<(), OutputErrorKind>,
) {
    let judged = judged(marked(files), &target, REPLACE).map(drop);
    assert_eq!(judged, outcome);
}

// Rule 122.
#[test]
fn a_version_whose_folder_is_missing_stops_every_command() {
    let files: Vec<&str> = VERSIONS
        .iter()
        .copied()
        .filter(|file| !file.starts_with("v1.0.0/"))
        .collect();
    let folders: Vec<&str> = vec!["data", "data/versions", "main"];
    let found: Vec<&str> = files.iter().copied().chain(["ghg-site.json"]).collect();
    let contents = || folder(&found, &folders, Some(marker(&VERSIONS)));
    let missing = Err(OutputErrorKind::MissingVersion {
        name: "v1.0.0".into(),
    });
    assert_eq!(judged(contents(), &version("v2"), REPLACE), missing);
    let removing = output_safety(
        contents(),
        &version("main"),
        Change::Remove,
        REPLACE,
        &KeptInOutput::default(),
    );
    assert_eq!(removing, missing);
}

// Rules 38 and 123: a site or a version already built is replaced only when allowed, and is
// checked again only before the input is read.
#[rstest]
#[case::a_site_kept(&SITE, BuildTarget::Site, KEEP, Err(OutputErrorKind::AlreadyBuilt))]
#[case::a_site_replaced(&SITE, BuildTarget::Site, REPLACE, Ok(()))]
#[case::a_version_kept(&VERSIONS, version("main"), KEEP, Err(OutputErrorKind::AlreadyBuilt))]
#[case::a_version_replaced(&VERSIONS, version("main"), REPLACE, Ok(()))]
#[case::a_new_version_kept(&VERSIONS, version("v2"), KEEP, Ok(()))]
#[case::a_version_written(&VERSIONS, version("main"), WRITING, Ok(()))]
#[case::a_preview_always_replaced(&PREVIEW, preview("pr-1"), KEEP, Ok(()))]
fn what_is_built_already_is_replaced_only_when_allowed(
    #[case] files: &[&str],
    #[case] target: BuildTarget,
    #[case] moment: Moment<'_>,
    #[case] outcome: Result<(), OutputErrorKind>,
) {
    assert_eq!(judged(marked(files), &target, moment).map(drop), outcome);
}

// Rules 257 and 258: a removal is judged by the marker alone, whatever the store holds beside.
#[rstest]
#[case::a_version_beside_a_site(&SITE, version("main"))]
#[case::a_preview_of_other_capitals(&PREVIEW, preview("PR-1"))]
fn a_removal_is_not_held_to_what_the_store_holds(
    #[case] files: &[&str],
    #[case] target: BuildTarget,
) {
    let judged = output_safety(
        marked(files),
        &target,
        Change::Remove,
        REPLACE,
        &KeptInOutput::default(),
    );
    assert!(judged.is_ok(), "{judged:?}");
}

#[rstest]
#[case::at_the_root("index.html", &[])]
#[case::two_folders_down("data/objects/a.json", &["data", "data/objects"])]
fn the_folders_of_a_file_are_listed_outermost_first(#[case] file: &str, #[case] folders: &[&str]) {
    assert_eq!(folders_of(file).collect::<Vec<_>>(), folders);
}

/// A folder holding `files` and `folders`, and `kept`, what the patterns keep, with no marker.
fn holding_kept(files: &[&str], folders: &[&str], kept: &[(&str, EntryKind)]) -> OutputContents {
    OutputContents::Folder {
        files: set(files),
        folders: set(folders),
        kept: kept
            .iter()
            .map(|(path, kind)| (path.to_string(), *kind))
            .collect(),
        marker: None,
    }
}

// Rule 227: what is kept counts as nothing in the folder, nor do the folders that hold it.
#[rstest]
#[case::a_kept_file(holding_kept(&[], &[], &[(".nojekyll", EntryKind::File)]))]
#[case::a_kept_file_in_a_folder(holding_kept(&[], &["docs"], &[("docs/CNAME", EntryKind::File)]))]
fn what_is_kept_counts_as_nothing_in_the_folder(#[case] contents: OutputContents) {
    let judged = output_safety(
        contents,
        &BuildTarget::Site,
        Change::Add,
        REPLACE,
        &kept_in_output(&[".nojekyll", "CNAME"]),
    );
    assert_eq!(judged, Ok(SafeFolder::default()));
}

// Rule 227: a marked folder holding what is kept beside what ghg wrote is still safe.
#[test]
fn a_marked_folder_holding_what_is_kept_is_safe() {
    let OutputContents::Folder {
        files,
        folders,
        marker,
        ..
    } = marked(&SITE)
    else {
        panic!("a folder");
    };
    let contents = OutputContents::Folder {
        files,
        folders,
        kept: BTreeMap::from([(".git".to_string(), EntryKind::Folder)]),
        marker,
    };
    let judged = output_safety(
        contents,
        &BuildTarget::Site,
        Change::Add,
        REPLACE,
        &kept_in_output(&[".git"]),
    );
    assert_eq!(judged.map(|safe| safe.listed), Ok(set(&SITE)));
}
