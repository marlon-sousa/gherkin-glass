mod against_fixtures;

use std::collections::BTreeSet;
use std::path::PathBuf;

use rstest::rstest;

use super::*;
use crate::errors::{InputError, InputErrorKind};
use crate::test_support::MemoryFolders;

/// The errors of the input tree of the input `in` that `folders` holds, each as its path and
/// what is wrong, in the order they are reported.
fn errors_of(folders: MemoryFolders) -> Vec<(String, InputErrorKind)> {
    let input_tree = InputCollector::new(Arc::new(folders)).collect(Path::new("in"));
    let errors = input_tree.input_errors().expect("errors");
    errors.as_slice().iter().map(path_and_kind).collect()
}

fn path_and_kind(error: &InputError) -> (String, InputErrorKind) {
    (error.path.clone(), error.kind.clone())
}

#[test]
fn files_with_the_same_page_are_reported_in_the_order_of_their_paths() {
    // Rules 15 and 37: `A` sorts before `a.feature`.
    let mut folders = MemoryFolders::with_files(["in/A", "in/a.feature", "in/a.story"]);
    folders
        .files
        .insert(PathBuf::from("in/.ghg-ignore"), "!*\n".to_string());
    let same_page = InputErrorKind::SamePage {
        page: "a.html".into(),
        first: "A".into(),
    };
    assert_eq!(
        errors_of(folders),
        [
            ("a.feature".to_string(), same_page.clone()),
            ("a.story".to_string(), same_page)
        ]
    );
}

#[test]
fn the_errors_of_the_walk_and_of_the_pages_are_reported_together_sorted() {
    // Rule 15.
    let mut folders =
        MemoryFolders::with_files(["in/a.feature", "in/a.story", "in/locked/b.feature"]);
    folders.files.insert(
        PathBuf::from("in/.ghg-ignore"),
        "!*.story\na{\n".to_string(),
    );
    folders.unlistable.insert(PathBuf::from("in/locked"));
    let reported: Vec<_> = errors_of(folders).into_iter().map(is_same_page).collect();
    assert_eq!(
        reported,
        [
            (".ghg-ignore".to_string(), false),
            ("a.story".to_string(), true),
            ("locked".to_string(), false)
        ]
    );
}

/// The error's path, and whether its page is another file's.
fn is_same_page((path, kind): (String, InputErrorKind)) -> (String, bool) {
    (path, matches!(kind, InputErrorKind::SamePage { .. }))
}

#[test]
fn a_folder_the_rules_leave_out_is_not_listed() {
    // Rule 2: a folder left out is left out with everything in it, even one that cannot be
    // listed.
    let mut folders = MemoryFolders::with_files(["in/.hidden/a.feature", "in/b.feature"]);
    folders.unlistable.insert(PathBuf::from("in/.hidden"));
    let input_tree = InputCollector::new(Arc::new(folders)).collect(Path::new("in"));
    assert!(input_tree.input_errors().is_none());
    assert_eq!(input_tree.allowed_files(), [PathBuf::from("in/b.feature")]);
}

#[rstest]
// Rule 65: against the last build's tree and the input walked again, a file made that the rules
// allow, one removed and a `.ghg-ignore` call for a rebuild, and a file the rules leave out does
// not.
#[case::a_file_made("in/new.feature", true)]
#[case::a_file_removed("in/gone.feature", true)]
#[case::a_ghg_ignore_file("in/.ghg-ignore", true)]
#[case::a_file_left_out("in/notes.txt", false)]
fn changes_call_for_a_rebuild_judged_against_the_last_tree(
    #[case] changed: &str,
    #[case] rebuilds: bool,
) {
    let last_input = MemoryFolders::with_files(["in/gone.feature", "in/kept.feature"]);
    let last = InputCollector::new(Arc::new(last_input)).collect(Path::new("in"));
    let input = MemoryFolders::with_files(["in/kept.feature", "in/new.feature", "in/notes.txt"]);
    let input_collector = InputCollector::new(Arc::new(input));
    let paths = BTreeSet::from([PathBuf::from(changed)]);
    assert_eq!(input_collector.calls_for_rebuild(&last, &paths), rebuilds);
}
