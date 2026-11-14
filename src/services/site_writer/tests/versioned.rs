//! The site writer's versioned builds, over the fakes.

use rstest::rstest;

use super::*;
use crate::entities::{OutputContents, VersionListing};
use fakes::*;

#[test]
fn a_versioned_build_replaces_its_folder_and_the_page_and_records_every_version() {
    let folders = FakeFolders::holding(two_versions());
    let writer = Arc::new(FakeWriter::default());
    build(
        vec!["a.feature"],
        Some("v1.0.0"),
        folders.clone(),
        writer.clone(),
    )
    .expect("built");
    assert_eq!(
        folders.done(),
        [
            "remove site/v1.0.0",
            "remove site/index.html",
            "record site"
        ]
    );
    let written = written(&writer);
    assert_eq!(
        written.first().map(String::as_str),
        Some("site/v1.0.0/a.html")
    );
    assert!(written.contains(&"site/index.html".to_string()));
    assert!(written.contains(&"site/assets/gherkin-glass.css".to_string()));
    let Some(Record::Versions { page, versions, .. }) =
        folders.recorded.lock().expect("lock").clone()
    else {
        panic!("no versions recorded");
    };
    assert!(page.contains("index.html") && page.contains("assets/gherkin-glass.css"));
    assert_eq!(versions.get(&name("main")), Some(&set(&["index.html"])));
    assert!(versions[&name("v1.0.0")].contains("a.html"));
}

#[test]
fn a_page_that_cannot_be_written_leaves_the_versions_page_unwritten() {
    let folders = FakeFolders::holding(OutputContents::Missing);
    let writer = Arc::new(FakeWriter {
        fails: Some("b.html"),
        ..FakeWriter::default()
    });
    let outcome = build(
        vec!["a.feature", "b.feature"],
        Some("v1.0.0"),
        folders.clone(),
        writer.clone(),
    );
    assert!(
        matches!(&outcome, Err(BuildError::Output(error)) if error.to_string() == "site/v1.0.0/b.html: cannot be written: disk full"),
        "{outcome:?}"
    );
    assert_eq!(folders.done(), ["remove site/v1.0.0", "record site"]);
    assert_eq!(written(&writer), ["site/v1.0.0/a.html"]);
}

/// The output folder of `two_versions()`, its marker recording `previews` as previews.
fn two_versions_with_previews(previews: &[&str]) -> OutputContents {
    let mut contents = two_versions();
    if let OutputContents::Folder {
        record: Some(Record::Versions { previews: kept, .. }),
        ..
    } = &mut contents
    {
        *kept = previews.iter().map(|preview| name(preview)).collect();
    }
    contents
}

/// The previews the marker `folders` was given records.
fn recorded_previews(folders: &FakeFolders) -> Vec<String> {
    match folders.recorded.lock().expect("lock").clone() {
        Some(Record::Versions { previews, .. }) => {
            previews.iter().map(VersionName::to_string).collect()
        }
        record => panic!("{record:?}"),
    }
}

// Rule 212: the versions page lists every version but the previews.
#[test]
fn the_versions_checked_are_those_the_versions_page_lists_every_one_but_the_previews() {
    let folders = FakeFolders::holding(two_versions_with_previews(&["v1.0.0"]));
    let writer = SiteWriter::new(
        Arc::new(FakeTemplate::default()),
        folders,
        Arc::new(FakeWriter::default()),
        Arc::new(FakeWriter::default()),
    );
    let listed = writer
        .check(&settings(Destination::Folder, Some("pr-1")))
        .expect("safe");
    assert_eq!(listed, BTreeSet::from([name("main")]));
}

// Rule 212: a preview is recorded as one, and a version built again without
// --preview is listed again.
#[rstest]
#[case::a_preview_is_recorded(&[], "pr-1", VersionListing::Preview, &["pr-1"])]
#[case::a_preview_built_again_stays_one(&["pr-1"], "pr-1", VersionListing::Preview, &["pr-1"])]
#[case::other_previews_are_kept(&["v1.0.0"], "pr-1", VersionListing::Preview, &["pr-1", "v1.0.0"])]
#[case::a_preview_built_listed_is_listed_again(&["v1.0.0"], "v1.0.0", VersionListing::Listed, &[])]
fn the_marker_records_the_previews(
    #[case] before: &[&str],
    #[case] version: &str,
    #[case] listing: VersionListing,
    #[case] after: &[&str],
) {
    let folders = FakeFolders::holding(two_versions_with_previews(before));
    build_as(
        vec!["a.feature"],
        Some(version),
        listing,
        folders.clone(),
        Arc::new(FakeWriter::default()),
    )
    .expect("built");
    assert_eq!(recorded_previews(&folders), after);
}
