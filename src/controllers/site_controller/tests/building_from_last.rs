//! The controller's tests of a build from the last build, with the real services over an input
//! held in memory and the site written in memory: after every change of a sequence, the site
//! built from the last build and the paths that changed is the site built from nothing, byte for
//! byte.

mod changing_input;
mod random_changes;

use std::path::PathBuf;
use std::sync::Arc;

use rstest::rstest;

use super::super::building::{BuildOutcome, Built, Stopped};
use super::*;
use crate::adapters::MemoryFiles;
use crate::entities::{BuildSnapshot, BuildStart, PathList};
use crate::services::{Converter, GherkinParser, InputCollector, SettingsService, SiteWriter};
use changing_input::{Change, ChangingInput};

const TAGGED: &str = "@slow\nFeature: A\n  Scenario: S\n    Given a step\n";
const EDITED: &str =
    "@slow\nFeature: A\n  Scenario: S\n    Given a step\n\n  Scenario: T\n    Then another\n";
const PLAIN: &str = "Feature: B\n  Scenario: U\n    When b\n";
const RETAGGED: &str = "@Slow\nFeature: B\n  Scenario: U\n    When b\n";
const BROKEN: &str = "Feature: B\n  Scenario: U\n    When b\n  Rule\n";
const DRAFT: &str = "Feature: D\n  Scenario: V\n    Given d\n";

/// The input every sequence starts from: two features, one in a folder, a hidden folder the
/// defaults leave out, and a file they leave out.
const START: [Change; 4] = [
    Change::Write("in/a.feature", TAGGED),
    Change::Write("in/sub/b.feature", PLAIN),
    Change::Write("in/.drafts/d.feature", DRAFT),
    Change::Write("in/notes.txt", "notes"),
];

/// The site `outcome` served, or what stopped it.
fn site_of(outcome: &BuildOutcome) -> Result<&ServedSite, &BuildError> {
    match outcome {
        BuildOutcome::Built(Built { site, .. }) => Ok(site),
        BuildOutcome::Stopped(Stopped { error, .. }) => Err(error),
        other => panic!("built or stopped, not {other:?}"),
    }
}

/// What the next build starts from after `outcome`, or `last` when it left nothing.
fn left_by(outcome: BuildOutcome, last: BuildSnapshot) -> BuildSnapshot {
    match outcome {
        BuildOutcome::Built(Built { snapshot, .. }) => snapshot,
        BuildOutcome::Stopped(Stopped { left, .. }) => left.unwrap_or(last),
        BuildOutcome::NothingToBuild(left) | BuildOutcome::SettingsChanged { left, .. } => left,
    }
}

#[rstest]
// Rules 1 to 3, 15, 65 and 67: a rebuild from the last build parses again only what changed and
// still gives the site a build from nothing gives, or stops on the same errors.
#[case::editing_a_feature(&[Change::Write("in/a.feature", EDITED)])]
#[case::adding_a_tag_that_renames_another_tag_s_page(&[
    Change::Write("in/sub/b.feature", RETAGGED),
    Change::Write("in/sub/b.feature", PLAIN),
])]
#[case::adding_a_file_in_a_new_folder(&[Change::Write("in/new/deeper/c.feature", PLAIN)])]
#[case::allowing_a_folder_through_a_rule_and_forbidding_it_again(&[
    Change::Write("in/.ghg-ignore", "!.drafts/\n"),
    Change::Write("in/.drafts/d.feature", PLAIN),
    Change::Write("in/.ghg-ignore", "# nothing\n"),
    Change::Remove("in/.ghg-ignore"),
])]
#[case::leaving_out_a_folder_s_files_from_a_rule_below_it(&[
    Change::Write("in/sub/.ghg-ignore", "*.feature\n"),
    Change::Write("in/sub/.ghg-ignore", "!b.feature\n"),
])]
#[case::removing_a_file(&[Change::Remove("in/a.feature")])]
#[case::removing_a_folder(&[Change::Remove("in/sub")])]
#[case::renaming_a_folder(&[
    Change::Rename("in/sub", "in/moved"),
    Change::Rename("in/moved", "in/sub"),
])]
#[case::replacing_a_folder_whose_path_alone_is_named(&[
    Change::Replace("in/sub", "b.feature", RETAGGED),
    Change::Replace("in", "a.feature", EDITED),
])]
#[case::a_broken_file_holding_back_a_change_until_it_is_fixed(&[
    Change::Write("in/sub/b.feature", BROKEN),
    Change::Write("in/a.feature", EDITED),
    Change::Write("in/sub/b.feature", PLAIN),
])]
#[case::a_file_the_rules_leave_out(&[
    Change::Write("in/notes.txt", "more notes"),
    Change::Write("in/a.feature", EDITED),
])]
fn a_build_from_the_last_build_gives_the_site_a_build_from_nothing_gives(
    #[case] changes: &[Change],
) {
    builds_from_the_last_build_match_builds_from_nothing(changes, None);
}

/// The list `--only` reads in the sequences of only some files: `a.feature` and a file made later.
const LIST: Change = Change::Write("list.txt", "in/a.feature\nin/sub/new.feature\n");

#[rstest]
// Rules 205, 206 and 209: with --only, a change to a listed file, to a file not listed, even one
// that cannot be parsed, and to the list itself.
#[case::a_listed_file_edited(&[Change::Write("in/a.feature", EDITED)])]
#[case::a_file_not_listed_broken(&[
    Change::Write("in/sub/b.feature", BROKEN),
    Change::Write("in/a.feature", EDITED),
])]
#[case::a_listed_file_made(&[Change::Write("in/sub/new.feature", PLAIN)])]
#[case::a_listed_file_removed(&[Change::Remove("in/a.feature")])]
#[case::the_list_changed(&[Change::Write("list.txt", "in/sub/b.feature\n")])]
fn a_build_of_the_listed_files_from_the_last_build_gives_the_site_a_build_from_nothing_gives(
    #[case] changes: &[Change],
) {
    builds_from_the_last_build_match_builds_from_nothing(
        changes,
        Some(PathList::File(PathBuf::from("list.txt"))),
    );
}

/// Builds the input `START` makes, with `LIST` beside it, then applies each of `changes`, the
/// site built from the files `only` lists, if any.
fn builds_from_the_last_build_match_builds_from_nothing(
    changes: &[Change],
    only: Option<PathList>,
) {
    let input = Arc::new(ChangingInput::default());
    START
        .iter()
        .chain([&LIST])
        .for_each(|change| input.apply(change));
    let controller = SiteController::new(
        Arc::new(SettingsService::new(input.clone(), input.clone())),
        Arc::new(InputCollector::new(input.clone())),
        Arc::new(GherkinParser::new(input.clone())),
        Arc::new(Converter),
        Arc::new(SiteWriter::new(
            input.clone(),
            input.clone(),
            Arc::new(MemoryFiles::default()),
            Arc::new(MemoryFiles::default()),
        )),
        world().server,
    );
    let options = BuildOptions {
        input: Some(PathBuf::from("in")),
        only,
        ..BuildOptions::default()
    };
    let settings = controller
        .settings
        .load(&options, Destination::Memory)
        .expect("the settings");
    let mut last = controller
        .build_from_nothing(settings.clone())
        .expect("the first site")
        .snapshot;
    let mut before = controller.build_from(
        BuildStart::FromNothing(Box::new(settings.clone())),
        &options,
    );
    for change in changes {
        input.apply(change);
        let paths = change.paths();
        let start = BuildStart::FromLast {
            last: &last,
            paths: &paths,
        };
        let from_last = controller.build_from(start, &options);
        let from_nothing = controller.build_from(
            BuildStart::FromNothing(Box::new(settings.clone())),
            &options,
        );
        let expected = match &from_last {
            BuildOutcome::NothingToBuild(_) => &before,
            built_or_stopped => built_or_stopped,
        };
        assert_eq!(site_of(&from_nothing), site_of(expected), "{change:?}");
        last = left_by(from_last, last);
        before = from_nothing;
    }
}
