mod against_fixtures;
mod fakes;
mod versioned;

use std::sync::Mutex;

use rstest::rstest;

use super::*;
use crate::entities::OutputContents;
use crate::errors::InputErrorKind;
use fakes::*;

#[rstest]
#[case::missing(OutputContents::Missing)]
#[case::empty(folder(&[], &[], None))]
#[case::a_site_ghg_built(folder(
    &["ghg-site.json", "old.html"],
    &[],
    Some(Record::Site(set(&["old.html"])))
))]
fn a_site_is_built_into_a_missing_empty_or_safe_folder_marked_with_its_files(
    #[case] contents: OutputContents,
) {
    let folders = FakeFolders::holding(contents);
    let writer = Arc::new(FakeWriter::default());
    build(vec!["a.feature"], None, folders.clone(), writer.clone()).expect("built");
    assert_eq!(folders.done(), ["clear site", "record site"]);
    let written = written(&writer);
    assert_eq!(written.first().map(String::as_str), Some("site/a.html"));
    let Some(Record::Site(files)) = folders.recorded.lock().expect("lock").clone() else {
        panic!("no site recorded");
    };
    let expected: BTreeSet<_> = written
        .iter()
        .filter_map(|path| path.strip_prefix("site/"))
        .map(str::to_string)
        .collect();
    assert_eq!(files, expected);
}

#[test]
fn input_with_an_error_leaves_an_earlier_site_as_it_was() {
    let folders = FakeFolders::holding(folder(
        &["ghg-site.json"],
        &[],
        Some(Record::Site(BTreeSet::new())),
    ));
    let writer = Arc::new(FakeWriter::default());
    let outcome = build(vec!["bad.feature"], None, folders.clone(), writer.clone());
    assert!(matches!(outcome, Err(BuildError::Input(_))));
    assert!(folders.done().is_empty());
    assert!(written(&writer).is_empty());
}

#[test]
fn a_page_of_a_site_that_cannot_be_written_is_reported_with_its_path() {
    let writer = Arc::new(FakeWriter {
        fails: Some("b.html"),
        ..FakeWriter::default()
    });
    let folders = FakeFolders::holding(OutputContents::Missing);
    let outcome = build(vec!["a.feature", "b.feature"], None, folders, writer);
    let Err(BuildError::Input(errors)) = outcome else {
        panic!("{outcome:?}");
    };
    let kinds: Vec<_> = errors
        .as_slice()
        .iter()
        .map(|error| (error.path.as_str(), &error.kind))
        .collect();
    let kind = InputErrorKind::CannotBeWritten {
        reason: "disk full".into(),
    };
    assert_eq!(kinds, [("site/b.html", &kind)]);
}

#[test]
fn an_output_folder_that_cannot_be_looked_into_is_reported_as_such() {
    let folders = FakeFolders::looking(vec![None]);
    let outcome = build(vec!["a.feature"], None, folders, Arc::default());
    assert!(matches!(
        outcome,
        Err(BuildError::Output(OutputError {
            kind: OutputErrorKind::CannotBeRead { .. },
            ..
        }))
    ));
}

#[test]
fn a_folder_that_changed_while_the_input_was_read_is_left_as_it_is() {
    let changed = folder(&["notes.txt"], &[], None);
    let folders = FakeFolders::looking(vec![Some(OutputContents::Missing), Some(changed)]);
    let writer = Arc::new(FakeWriter::default());
    let outcome = build(vec!["a.feature"], None, folders.clone(), writer.clone());
    assert!(matches!(
        outcome,
        Err(BuildError::Output(OutputError {
            kind: OutputErrorKind::NotASite,
            ..
        }))
    ));
    assert!(folders.done().is_empty());
    assert!(written(&writer).is_empty());
}

#[test]
fn a_marker_that_cannot_be_written_stops_the_build_before_any_page() {
    let folders = Arc::new(FakeFolders {
        looks: Mutex::new(vec![Some(OutputContents::Missing)]),
        fails: vec!["record"],
        ..FakeFolders::default()
    });
    let writer = Arc::new(FakeWriter::default());
    let outcome = build(vec!["a.feature"], None, folders, writer.clone());
    assert!(matches!(
        outcome,
        Err(BuildError::Output(OutputError {
            kind: OutputErrorKind::CannotBeWritten { .. },
            ..
        }))
    ));
    assert!(written(&writer).is_empty());
}
