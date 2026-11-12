mod against_fixtures;
mod fakes;
mod versioned;

use std::sync::Mutex;

use rstest::rstest;

use super::*;
use crate::adapters::MemoryFiles;
use crate::entities::OutputContents;
use crate::errors::BuildError;
use crate::services::{Converter, ConverterApi};
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
fn a_page_of_a_site_that_cannot_be_written_is_reported_with_its_path() {
    let writer = Arc::new(FakeWriter {
        fails: Some("b.html"),
        ..FakeWriter::default()
    });
    let folders = FakeFolders::holding(OutputContents::Missing);
    let outcome = build(vec!["a.feature", "b.feature"], None, folders, writer);
    let Err(BuildError::Output(error)) = outcome else {
        panic!("{outcome:?}");
    };
    assert_eq!(
        error.to_string(),
        "site/b.html: cannot be written: disk full"
    );
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

#[test]
fn a_site_in_memory_checks_no_folder_and_has_no_versions() {
    // Rule 66.
    let folders = FakeFolders::looking(vec![None]);
    let memory = Arc::new(MemoryFiles::default());
    memory
        .write(Path::new("site"), "stale.html", b"")
        .expect("written");
    let writer = SiteWriter::new(folders.clone(), Arc::new(FakeWriter::default()), memory);
    let settings = Settings {
        replace: false,
        ..settings(Destination::Memory, Some("v1.0.0"))
    };
    writer.check(&settings).expect("no check");
    let site = Converter.convert(&[], &settings);
    let (_, ServedSite::Memory(files)) = writer.write(&settings, &site).expect("written") else {
        panic!("a site in memory");
    };
    assert!(files.contains_key("index.html"));
    assert!(!files.contains_key("stale.html"));
    assert!(folders.done().is_empty());
}
