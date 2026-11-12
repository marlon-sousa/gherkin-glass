//! The controller's tests of a build from the last build, with the real services over an input
//! held in memory and the site written in memory: after every change of a sequence, the site
//! built from the last build and the paths that changed is the site built from nothing, byte for
//! byte.

use std::collections::BTreeSet;
use std::io::{Error as IoError, ErrorKind, Result as IoResult};
use std::mem::take;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use gherkin::{Feature, GherkinEnv};
use rstest::rstest;

use super::super::building::{BuildOutcome, Built, Stopped};
use super::*;
use crate::adapters::MemoryFiles;
use crate::entities::{BuildSnapshot, BuildStart, FolderEntry, OutputContents, RawFeature, Record};
use crate::errors::{InputErrorKind, ParseError};
use crate::ports::{ConfigReader, FeatureParser, InputFolders, OutputFolders};
use crate::services::{Converter, GherkinParser, InputCollector, SettingsService, SiteWriter};
use crate::test_support::MemoryFolders;

/// An input held in memory that a test changes between builds, read by the input collector and
/// parsed by the gherkin parser, with no configuration file and no output folder.
#[derive(Default)]
struct ChangingInput {
    folders: Mutex<MemoryFolders>,
}

impl ChangingInput {
    fn apply(&self, change: &Change) {
        let mut folders = self.folders.lock().unwrap_or_else(PoisonError::into_inner);
        match change {
            Change::Write(path, text) => {
                folders.files.insert(PathBuf::from(path), text.to_string());
            }
            Change::Remove(path) => folders.files.retain(|file, _| !file.starts_with(path)),
            Change::Rename(from, to) => {
                let (moved, kept) = take(&mut folders.files)
                    .into_iter()
                    .partition(|(file, _): &(PathBuf, String)| file.starts_with(from));
                folders.files = kept;
                for (file, text) in moved {
                    let below = file.strip_prefix(from).expect("in the folder moved");
                    folders.files.insert(Path::new(to).join(below), text);
                }
            }
            Change::Replace(folder, file, text) => {
                folders
                    .files
                    .insert(Path::new(folder).join(file), text.to_string());
            }
        }
    }
}

impl InputFolders for ChangingInput {
    fn list(&self, folder: &Path) -> IoResult<Vec<FolderEntry>> {
        let folders = self.folders.lock().unwrap_or_else(PoisonError::into_inner);
        folders.list(folder)
    }

    fn read(&self, file: &Path) -> IoResult<String> {
        let folders = self.folders.lock().unwrap_or_else(PoisonError::into_inner);
        folders.read(file)
    }
}

impl FeatureParser for ChangingInput {
    fn parse(&self, path: &Path) -> Result<RawFeature, ParseError> {
        let source = InputFolders::read(self, path).map_err(|error| ParseError {
            line: None,
            kind: InputErrorKind::CannotBeRead {
                reason: error.to_string(),
            },
        })?;
        let feature = Feature::parse(&source, GherkinEnv::default()).map_err(|_| ParseError {
            line: Some(1),
            kind: InputErrorKind::Syntax,
        })?;
        Ok(RawFeature { feature, source })
    }
}

impl ConfigReader for ChangingInput {
    fn read(&self, _: &Path) -> IoResult<String> {
        Err(not_found())
    }
}

/// A site built in memory touches no output folder, so there is none.
impl OutputFolders for ChangingInput {
    fn contents(&self, _: &Path) -> IoResult<OutputContents> {
        Err(not_found())
    }

    fn clear(&self, _: &Path) -> IoResult<()> {
        Err(not_found())
    }

    fn remove(&self, _: &Path) -> IoResult<()> {
        Err(not_found())
    }

    fn record(&self, _: &Path, _: &Record) -> IoResult<()> {
        Err(not_found())
    }
}

fn not_found() -> IoError {
    IoError::new(ErrorKind::NotFound, "not found")
}

/// A change to the input, which one burst tells of.
#[derive(Debug)]
enum Change {
    /// A file made or changed, with its text, by its path.
    Write(&'static str, &'static str),
    /// A file removed, or a folder with everything in it, by its path.
    Remove(&'static str),
    /// A folder moved, with everything in it, by both its paths.
    Rename(&'static str, &'static str),
    /// A file made or changed in a folder, with its text, by the folder's path alone, as a
    /// watcher tells of a folder put in place of another.
    Replace(&'static str, &'static str, &'static str),
}

impl Change {
    /// The paths the burst names.
    fn paths(&self) -> BTreeSet<PathBuf> {
        let paths = match self {
            Change::Write(path, _) | Change::Remove(path) | Change::Replace(path, ..) => {
                vec![path]
            }
            Change::Rename(from, to) => vec![from, to],
        };
        paths.into_iter().map(PathBuf::from).collect()
    }
}

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
    let input = Arc::new(ChangingInput::default());
    START.iter().for_each(|change| input.apply(change));
    let controller = SiteController::new(
        Arc::new(SettingsService::new(input.clone())),
        Arc::new(InputCollector::new(input.clone())),
        Arc::new(GherkinParser::new(input.clone())),
        Arc::new(Converter),
        Arc::new(SiteWriter::new(
            input.clone(),
            Arc::new(MemoryFiles::default()),
            Arc::new(MemoryFiles::default()),
        )),
        world().server,
    );
    let options = BuildOptions {
        input: Some(PathBuf::from("in")),
        ..BuildOptions::default()
    };
    let settings = controller
        .settings
        .load(&options, Destination::Memory)
        .expect("the settings");
    let first = controller
        .build_from_nothing(settings.clone())
        .expect("the first site");
    let mut served = first.site;
    let mut last = first.snapshot;
    for change in changes {
        input.apply(change);
        let paths = change.paths();
        let start = BuildStart::FromLast {
            last: &last,
            paths: &paths,
        };
        let from_last = controller.build_from(start, &options);
        let from_nothing =
            controller.build_from(BuildStart::FromNothing(settings.clone()), &options);
        match &from_last {
            BuildOutcome::NothingToBuild(_) => {
                assert_eq!(site_of(&from_nothing), Ok(&served), "{change:?}");
            }
            built_or_stopped => {
                assert_eq!(
                    site_of(built_or_stopped),
                    site_of(&from_nothing),
                    "{change:?}"
                );
                if let Ok(site) = site_of(built_or_stopped) {
                    served = site.clone();
                }
            }
        }
        last = left_by(from_last, last);
    }
}
