//! The property of the named sequences, checked over random sequences of changes drawn from a
//! fixed set of paths and texts.

use proptest::collection::vec;
use proptest::prelude::{Just, Strategy, prop_oneof, proptest};
use proptest::sample::select;

use super::*;

const FEATURE_FILES: [&str; 4] = [
    "in/a.feature",
    "in/sub/b.feature",
    "in/new/c.feature",
    "in/.drafts/d.feature",
];
const FEATURE_TEXTS: [&str; 6] = [TAGGED, EDITED, PLAIN, RETAGGED, BROKEN, DRAFT];
const RULES_FILES: [&str; 2] = ["in/.ghg-ignore", "in/sub/.ghg-ignore"];
const RULES: [&str; 4] = ["!.drafts/\n", "*.feature\n", "!b.feature\n", "# nothing\n"];
const FOLDERS: [&str; 4] = ["in/sub", "in/new", "in/.drafts", "in/moved"];
/// The folders a file is put in by the folder's path alone, the input folder among them.
const HOLDERS: [&str; 3] = ["in", "in/sub", "in/new"];
const FILE_NAMES: [&str; 3] = ["a.feature", "b.feature", "c.feature"];
/// Files and folders, each removed with everything in it.
const REMOVED: [&str; 7] = [
    "in/sub",
    "in/new",
    "in/.drafts",
    "in/a.feature",
    "in/sub/b.feature",
    "in/.ghg-ignore",
    "in/notes.txt",
];

fn written_feature() -> impl Strategy<Value = Change> {
    (select(&FEATURE_FILES[..]), select(&FEATURE_TEXTS[..]))
        .prop_map(|(path, text)| Change::Write(path, text))
}

fn written_rules() -> impl Strategy<Value = Change> {
    (select(&RULES_FILES[..]), select(&RULES[..]))
        .prop_map(|(path, text)| Change::Write(path, text))
}

fn removed() -> impl Strategy<Value = Change> {
    select(&REMOVED[..]).prop_map(Change::Remove)
}

fn renamed() -> impl Strategy<Value = Change> {
    (select(&FOLDERS[..]), select(&FOLDERS[..])).prop_map(|(from, to)| Change::Rename(from, to))
}

fn replaced() -> impl Strategy<Value = Change> {
    (
        select(&HOLDERS[..]),
        select(&FILE_NAMES[..]),
        select(&FEATURE_TEXTS[..]),
    )
        .prop_map(|(folder, file, text)| Change::Replace(folder, file, text))
}

fn change() -> impl Strategy<Value = Change> {
    prop_oneof![
        3 => written_feature(),
        2 => written_rules(),
        1 => Just(Change::Write("in/notes.txt", "more notes")),
        2 => removed(),
        1 => renamed(),
        1 => replaced(),
    ]
}

proptest! {
    #[test]
    fn a_build_from_the_last_build_gives_the_site_a_build_from_nothing_gives_after_any_changes(
        changes in vec(change(), 1..8),
    ) {
        builds_from_the_last_build_match_builds_from_nothing(&changes);
    }
}
