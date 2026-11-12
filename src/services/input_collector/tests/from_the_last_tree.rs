//! The input collector's tests of an input tree made from the last one and the paths that
//! changed.

use std::ptr::eq;

use super::*;
use crate::entities::{FileToParse, ParsedFile};
use crate::test_support::parsed_at;

/// The tree of the input `in` that `folders` holds, every allowed file parsed.
fn parsed_tree(folders: MemoryFolders) -> InputTree {
    let mut input_tree = InputCollector::new(Arc::new(folders)).collect(Path::new("in"));
    let parsed = input_tree
        .files_to_parse()
        .into_iter()
        .map(parsed_file)
        .collect();
    input_tree.put_parsed(parsed);
    input_tree
}

fn parsed_file(file: FileToParse) -> ParsedFile {
    let result = Ok(parsed_at(&file.path.display().to_string(), "Feature: F\n"));
    ParsedFile { file, result }
}

fn changed(paths: &[&str]) -> BTreeSet<PathBuf> {
    paths.iter().map(PathBuf::from).collect()
}

fn to_parse(input_tree: &InputTree) -> Vec<PathBuf> {
    input_tree
        .files_to_parse()
        .into_iter()
        .map(|file| file.path)
        .collect()
}

#[test]
fn only_the_files_the_paths_name_are_parsed_again() {
    let files = ["in/a.feature", "in/b.feature", "in/sub/c.feature"];
    let last = parsed_tree(MemoryFolders::with_files(files));
    let input_collector = InputCollector::new(Arc::new(MemoryFolders::with_files(files)));
    let changes = input_collector.collect_changes(&last, &changed(&["in/b.feature"]));
    assert!(changes.calls_for_rebuild);
    assert_eq!(
        to_parse(&changes.input_tree),
        [PathBuf::from("in/b.feature")]
    );
    assert_eq!(changes.input_tree.features().count(), 2);
    // What parsing gave the files kept is shared with the last tree, not copied.
    let shared = changes
        .input_tree
        .features()
        .filter(|new| last.features().any(|last| eq(last.feature, new.feature)))
        .count();
    assert_eq!(shared, 2);
}

#[test]
fn a_folder_no_path_concerns_is_copied_without_being_listed_again() {
    let last = parsed_tree(MemoryFolders::with_files([
        "in/a.feature",
        "in/sub/c.feature",
    ]));
    let mut folders = MemoryFolders::with_files(["in/a.feature", "in/sub/c.feature"]);
    folders.unlistable.insert(PathBuf::from("in/sub"));
    let input_collector = InputCollector::new(Arc::new(folders));
    let changes = input_collector.collect_changes(&last, &changed(&["in/a.feature"]));
    assert!(changes.input_tree.input_errors().is_none());
    assert_eq!(
        changes.input_tree.allowed_files(),
        [
            PathBuf::from("in/a.feature"),
            PathBuf::from("in/sub/c.feature")
        ]
    );
}

#[test]
fn a_ghg_ignore_judges_again_its_folder_and_below_and_marks_what_it_leaves_out() {
    // Rules 2 and 3: the rules of `sub/.ghg-ignore` apply in `sub` and below, and nowhere else.
    let files = [
        "in/a.feature",
        "in/sub/c.feature",
        "in/sub/deeper/d.feature",
    ];
    let last = parsed_tree(MemoryFolders::with_files(files));
    let mut folders = MemoryFolders::with_files(files);
    folders.files.insert(
        PathBuf::from("in/sub/.ghg-ignore"),
        "c.feature\ndeeper/\n".into(),
    );
    let input_collector = InputCollector::new(Arc::new(folders));
    let changes = input_collector.collect_changes(&last, &changed(&["in/sub/.ghg-ignore"]));
    assert!(changes.calls_for_rebuild);
    assert_eq!(
        changes.input_tree.allowed_files(),
        [PathBuf::from("in/a.feature")]
    );
    assert!(to_parse(&changes.input_tree).is_empty());
}

#[test]
fn paths_outside_the_input_leave_the_tree_as_it_was() {
    let last = parsed_tree(MemoryFolders::with_files(["in/a.feature"]));
    let input_collector = InputCollector::new(Arc::new(MemoryFolders::default()));
    let changes = input_collector.collect_changes(&last, &changed(&["elsewhere/a.feature"]));
    assert!(!changes.calls_for_rebuild);
    assert_eq!(
        changes.input_tree.allowed_files(),
        [PathBuf::from("in/a.feature")]
    );
    assert!(to_parse(&changes.input_tree).is_empty());
}
