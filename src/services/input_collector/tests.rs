mod against_fixtures;
mod from_the_last_tree;

use std::collections::BTreeSet;
use std::path::PathBuf;

use rstest::rstest;

use super::*;
use crate::entities::PathList;
use crate::errors::{InputError, InputErrorKind};
use crate::test_support::{MemoryFolders, input_collector, listing_collector};

/// The errors of the input tree of the input `in` that `folders` holds, each as its path and
/// what is wrong, in the order they are reported.
fn errors_of(folders: MemoryFolders) -> Vec<(String, InputErrorKind)> {
    let input_tree = input_collector(Arc::new(folders))
        .collect(Path::new("in"), None)
        .expect("no list");
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
    let input_tree = input_collector(Arc::new(folders))
        .collect(Path::new("in"), None)
        .expect("no list");
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
    let last = input_collector(Arc::new(last_input))
        .collect(Path::new("in"), None)
        .expect("no list");
    let input = MemoryFolders::with_files(["in/kept.feature", "in/new.feature", "in/notes.txt"]);
    let input_collector = input_collector(Arc::new(input));
    let paths = BTreeSet::from([PathBuf::from(changed)]);
    let changes = input_collector.collect_changes(&last, &paths);
    assert_eq!(changes.calls_for_rebuild, rebuilds);
}

/// The list `--only` gives, of `paths`, one on each line.
fn listed(paths: &[&str]) -> String {
    paths.iter().map(|path| format!("{path}\n")).collect()
}

#[rstest]
// Rules 205 and 206: a listed path is matched once made absolute, a path outside the input is
// passed over, and a folder with no listed file below it is not entered; rule 204: the list is
// read from its file or standard input.
#[case::a_file(PathList::File(PathBuf::from("changed.txt")))]
#[case::standard_input(PathList::StandardInput)]
fn only_the_listed_files_of_the_input_are_collected_and_no_other_folder_is_listed(
    #[case] list: PathList,
) {
    let mut folders = MemoryFolders::with_files([
        "in/a.feature",
        "in/sub/b.feature",
        "in/sub/c.feature",
        "in/other/d.feature",
    ]);
    folders.unlistable.insert(PathBuf::from("in/other"));
    let only = listed(&["in/a.feature", "./in/x/../sub/b.feature", "src/main.rs"]);
    let input_tree = listing_collector(Arc::new(folders), Some(&only))
        .collect(Path::new("in"), Some(&list))
        .expect("the list is read");
    assert!(input_tree.input_errors().is_none());
    assert_eq!(
        input_tree.allowed_files(),
        [
            PathBuf::from("in/a.feature"),
            PathBuf::from("in/sub/b.feature")
        ]
    );
}

#[rstest]
// Rule 204: named as it was given, `-` for standard input, a list that cannot be read and one
// that names no path alike.
#[case::a_file_that_does_not_exist(
    PathList::File(PathBuf::from("changed.txt")),
    None,
    "changed.txt: cannot be read: not found\n"
)]
#[case::standard_input_that_is_not_text(
    PathList::StandardInput,
    None,
    "-: cannot be read: not UTF-8\n"
)]
#[case::a_file_of_empty_lines(
    PathList::File(PathBuf::from("changed.txt")),
    Some("\n\n"),
    "changed.txt: names no path, so there is nothing to build.\n"
)]
#[case::a_file_of_blank_lines(
    PathList::File(PathBuf::from("changed.txt")),
    Some("  \n\t\r\n"),
    "changed.txt: names no path, so there is nothing to build.\n"
)]
#[case::nothing_on_standard_input(
    PathList::StandardInput,
    Some(""),
    "-: names no path, so there is nothing to build.\n"
)]
fn a_list_that_cannot_be_read_or_names_no_path_stops_the_run(
    #[case] list: PathList,
    #[case] text: Option<&str>,
    #[case] report: &str,
) {
    let folders = Arc::new(MemoryFolders::with_files(["in/a.feature"]));
    let collector = listing_collector(folders, text);
    let collected = collector.collect(Path::new("in"), Some(&list));
    assert_eq!(
        collected.map_err(|errors| errors.to_string()).err(),
        Some(report.to_string())
    );
}

#[rstest]
// Rule 209: a change to a file the list does not name rebuilds nothing, and the files are those
// the last tree was walked for, the list not being read again.
#[case::a_listed_file("in/a.feature", true)]
#[case::a_file_not_listed("in/b.feature", false)]
fn only_a_change_to_a_listed_file_calls_for_a_rebuild(
    #[case] changed: &str,
    #[case] rebuilds: bool,
) {
    let only = listed(&["in/a.feature"]);
    let folders = || Arc::new(MemoryFolders::with_files(["in/a.feature", "in/b.feature"]));
    let last = listing_collector(folders(), Some(&only))
        .collect(Path::new("in"), Some(&PathList::StandardInput))
        .expect("the list is read");
    let paths = BTreeSet::from([PathBuf::from(changed)]);
    // A collector whose list cannot be read, which collect_changes never reads.
    let changes = input_collector(folders()).collect_changes(&last, &paths);
    assert_eq!(changes.calls_for_rebuild, rebuilds);
    assert_eq!(
        changes.input_tree.allowed_files(),
        [PathBuf::from("in/a.feature")]
    );
}
