mod against_fixtures;

use std::collections::BTreeMap;
use std::io::{ErrorKind, Result as IoResult};
use std::path::PathBuf;
use std::sync::Mutex;

use gherkin::{Feature, GherkinEnv};

use super::*;
use crate::entities::{CollectedInput, PageOptions, RawFeature, Unnamed};
use crate::errors::ParseError;
use crate::ports::{FeatureParser, InputCollector, SiteWriter};
use crate::services::InputService;

/// Folders held in memory, by path, each with its state; a path held
/// with `None` cannot be read. Each operation named in `fails` fails.
#[derive(Default)]
struct FakeFolders {
    states: Mutex<BTreeMap<PathBuf, Option<FolderState>>>,
    done: Mutex<Vec<String>>,
    fails: Vec<&'static str>,
}

impl FakeFolders {
    fn with(states: &[(&str, Option<FolderState>)]) -> Arc<Self> {
        Arc::new(Self::holding(states, Vec::new()))
    }

    fn holding(states: &[(&str, Option<FolderState>)], fails: Vec<&'static str>) -> Self {
        let folders = FakeFolders {
            fails,
            ..FakeFolders::default()
        };
        *folders.states.lock().expect("lock") = states
            .iter()
            .map(|(path, state)| (PathBuf::from(path), *state))
            .collect();
        folders
    }

    fn record(&self, operation: &str, path: &Path) -> IoResult<()> {
        if self.fails.contains(&operation) {
            return Err(IoError::new(ErrorKind::PermissionDenied, "denied"));
        }
        self.done
            .lock()
            .expect("lock")
            .push(format!("{operation} {}", path.display()));
        Ok(())
    }
}

impl OutputFolders for FakeFolders {
    fn state(&self, folder: &Path, _marker: &Marker) -> IoResult<FolderState> {
        match self.states.lock().expect("lock").get(folder) {
            None => Ok(FolderState::Missing),
            Some(Some(state)) => Ok(*state),
            Some(None) => Err(IoError::new(ErrorKind::PermissionDenied, "denied")),
        }
    }

    fn clear(&self, folder: &Path) -> IoResult<()> {
        self.record("clear", folder)
    }

    fn remove(&self, path: &Path) -> IoResult<()> {
        self.record("remove", path)
    }

    fn mark(&self, folder: &Path, _marker: &Marker) -> IoResult<()> {
        self.record("mark", folder)?;
        self.states
            .lock()
            .expect("lock")
            .insert(folder.to_path_buf(), Some(FolderState::Marked));
        Ok(())
    }

    fn marked(&self, output: &Path) -> IoResult<Vec<VersionName>> {
        self.record("list", output)?;
        Ok(self
            .states
            .lock()
            .expect("lock")
            .iter()
            .filter(|(_, state)| **state == Some(FolderState::Marked))
            .filter(|(path, _)| path.parent() == Some(output))
            .filter_map(|(path, _)| VersionName::new(path.file_name()?.to_str()?))
            .collect())
    }
}

fn name(text: &str) -> VersionName {
    VersionName::new(text).expect("a version name")
}

fn check(states: &[(&str, Option<FolderState>)], replace: bool) -> Result<PathBuf, OutputError> {
    OutputService::new(FakeFolders::with(states))
        .check(Path::new("site"), &name("v1"), replace)
        .map(|folder| folder.path().to_path_buf())
}

#[test]
fn a_new_or_empty_folder_is_used() {
    assert_eq!(check(&[], false), Ok(PathBuf::from("site/v1")));
    let empty = [("site/v1", Some(FolderState::Empty))];
    assert_eq!(check(&empty, false), Ok(PathBuf::from("site/v1")));
}

#[test]
fn a_version_already_built_is_replaced_only_when_asked() {
    let built = [("site/v1", Some(FolderState::Marked))];
    assert_eq!(check(&built, true), Ok(PathBuf::from("site/v1")));
    assert_eq!(
        check(&built, false),
        Err(OutputError {
            output: PathBuf::from("site"),
            version: Some(name("v1")),
            kind: OutputErrorKind::AlreadyBuilt,
        })
    );
}

#[test]
fn a_folder_that_is_not_the_version_is_never_replaced() {
    for (state, kind) in [
        (FolderState::Other, OutputErrorKind::NotAVersion),
        (FolderState::OtherCapitals, OutputErrorKind::OtherCapitals),
    ] {
        for replace in [true, false] {
            assert_eq!(
                check(&[("site/v1", Some(state))], replace).map_err(|error| error.kind),
                Err(kind.clone())
            );
        }
    }
    let unreadable = [("site/v1", None)];
    assert!(matches!(
        check(&unreadable, true).map_err(|error| error.kind),
        Err(OutputErrorKind::CannotBeRead { .. })
    ));
}

#[test]
fn a_folder_that_cannot_be_emptied_or_listed_is_reported() {
    let folder = BuildFolder::new(Path::new("site"), Marker::Version(name("v1")));
    let service = OutputService::new(Arc::new(FakeFolders::holding(&[], vec!["clear"])));
    assert!(matches!(
        service.make_room(Path::new("site"), &folder),
        Err(OutputError {
            version: Some(_),
            kind: OutputErrorKind::CannotBeWritten { .. },
            ..
        })
    ));
    let service = OutputService::new(Arc::new(FakeFolders::holding(&[], vec!["list"])));
    assert!(matches!(
        service.versions(Path::new("site")),
        Err(OutputError {
            version: None,
            kind: OutputErrorKind::CannotBeListed { .. },
            ..
        })
    ));
}

struct FakeCollector(Vec<&'static str>);

impl InputCollector for FakeCollector {
    fn collect(&self, input: &Path) -> CollectedInput {
        CollectedInput {
            paths: self.0.iter().map(|path| input.join(path)).collect(),
            errors: Vec::new(),
        }
    }
}

/// Parses every file as a feature named after it, but fails `bad.feature`.
struct FakeParser;

impl FeatureParser for FakeParser {
    fn parse(&self, path: &Path) -> Result<RawFeature, ParseError> {
        if path.ends_with("bad.feature") {
            return Err(ParseError {
                line: Some(1),
                kind: InputErrorKind::Syntax,
            });
        }
        let source = format!("Feature: {}\n", path.display());
        let feature = Feature::parse(&source, GherkinEnv::default()).expect("parses");
        Ok(RawFeature { feature, source })
    }
}

/// Records every file written, by its folder and path, and fails `fails`.
#[derive(Default)]
struct FakeWriter {
    written: Mutex<Vec<String>>,
    fails: Option<&'static str>,
}

impl SiteWriter for FakeWriter {
    fn write(&self, output: &Path, file: &str, _contents: &[u8]) -> IoResult<()> {
        if self.fails == Some(file) {
            return Err(IoError::other("disk full"));
        }
        let path = format!("{}/{file}", output.display());
        self.written.lock().expect("lock").push(path);
        Ok(())
    }
}

fn build(
    files: Vec<&'static str>,
    folders: Arc<FakeFolders>,
    writer: Arc<FakeWriter>,
) -> Result<BuiltSite, BuildError> {
    let input = InputService::new(Arc::new(FakeCollector(files)), Arc::new(FakeParser));
    let site = SiteService::new(input, writer);
    let settings = Settings {
        input: PathBuf::from("in"),
        output: PathBuf::from("site"),
        unnamed: Unnamed::Allowed,
        pages: PageOptions::default(),
        version: None,
        replace: true,
    };
    OutputService::new(folders).build_version(&site, &settings, &name("v1.0.0"))
}

fn build_site(
    files: Vec<&'static str>,
    folders: Arc<FakeFolders>,
    writer: Arc<FakeWriter>,
) -> Result<BuiltSite, BuildError> {
    let input = InputService::new(Arc::new(FakeCollector(files)), Arc::new(FakeParser));
    let site = SiteService::new(input, writer);
    let settings = Settings {
        input: PathBuf::from("in"),
        output: PathBuf::from("site"),
        unnamed: Unnamed::Allowed,
        pages: PageOptions::default(),
        version: None,
        replace: true,
    };
    OutputService::new(folders).build_site(&site, &settings)
}

#[test]
fn a_site_is_built_into_an_empty_or_marked_folder_and_marked() {
    for state in [None, Some(FolderState::Empty), Some(FolderState::Marked)] {
        let states: Vec<_> = state
            .map(|state| ("site", Some(state)))
            .into_iter()
            .collect();
        let folders = FakeFolders::with(&states);
        let writer = Arc::new(FakeWriter::default());
        build_site(vec!["a.feature"], folders.clone(), writer.clone()).expect("built");
        assert_eq!(
            *folders.done.lock().expect("lock"),
            ["list site", "clear site", "mark site"]
        );
        assert_eq!(
            writer
                .written
                .lock()
                .expect("lock")
                .first()
                .map(String::as_str),
            Some("site/a.html")
        );
    }
}

#[test]
fn input_with_an_error_leaves_an_earlier_site_as_it_was() {
    let folders = FakeFolders::with(&[("site", Some(FolderState::Marked))]);
    let writer = Arc::new(FakeWriter::default());
    let outcome = build_site(vec!["bad.feature"], folders.clone(), writer.clone());
    assert!(matches!(outcome, Err(BuildError::Input(_))));
    assert_eq!(*folders.done.lock().expect("lock"), ["list site"]);
    assert!(writer.written.lock().expect("lock").is_empty());
}

#[test]
fn a_page_of_a_site_that_cannot_be_written_is_reported_with_its_path() {
    let writer = Arc::new(FakeWriter {
        fails: Some("b.html"),
        ..FakeWriter::default()
    });
    let outcome = build_site(
        vec!["a.feature", "b.feature"],
        FakeFolders::with(&[]),
        writer,
    );
    match outcome {
        Err(BuildError::Input(errors)) => assert_eq!(
            errors.to_string(),
            "site/b.html: cannot be written: disk full\n"
        ),
        other => panic!("{other:?}"),
    }
}

#[test]
fn an_output_folder_that_cannot_be_looked_into_is_reported_as_such() {
    let folders = FakeFolders::holding(&[], vec!["list"]);
    let error = OutputService::new(Arc::new(folders))
        .check_site(Path::new("site"), true)
        .expect_err("unreadable");
    assert!(matches!(error.kind, OutputErrorKind::CannotBeRead { .. }));
}

#[test]
fn a_folder_that_changed_since_its_check_is_not_emptied() {
    let folders = FakeFolders::with(&[("site", Some(FolderState::Other))]);
    let service = OutputService::new(folders.clone());
    let folder = BuildFolder::new(Path::new("site"), Marker::Site);
    let error = service
        .make_room(Path::new("site"), &folder)
        .expect_err("changed");
    assert_eq!(error.kind, OutputErrorKind::NotASite);
    assert!(folders.done.lock().expect("lock").is_empty());
}

#[test]
fn a_versioned_build_empties_marks_writes_and_then_writes_the_page() {
    let folders = FakeFolders::with(&[("site/main", Some(FolderState::Marked))]);
    let writer = Arc::new(FakeWriter::default());
    build(vec!["a.feature"], folders.clone(), writer.clone()).expect("built");
    assert_eq!(
        *folders.done.lock().expect("lock"),
        [
            "clear site/v1.0.0",
            "mark site/v1.0.0",
            "list site",
            "remove site/index.html",
            "remove site/assets",
        ]
    );
    let written = writer.written.lock().expect("lock");
    assert_eq!(
        written.first().map(String::as_str),
        Some("site/v1.0.0/a.html")
    );
    assert!(written.contains(&"site/index.html".to_string()));
    assert!(written.contains(&"site/assets/gherkin-glass.css".to_string()));
}

#[test]
fn input_with_an_error_empties_nothing_and_writes_nothing() {
    let folders = FakeFolders::with(&[("site/v1.0.0", Some(FolderState::Marked))]);
    let writer = Arc::new(FakeWriter::default());
    let outcome = build(vec!["bad.feature"], folders.clone(), writer.clone());
    assert!(matches!(outcome, Err(BuildError::Input(_))));
    assert!(folders.done.lock().expect("lock").is_empty());
    assert!(writer.written.lock().expect("lock").is_empty());
}

#[test]
fn a_page_that_cannot_be_written_leaves_the_version_marked_and_the_root_as_it_was() {
    let folders = FakeFolders::with(&[]);
    let writer = Arc::new(FakeWriter {
        fails: Some("b.html"),
        ..FakeWriter::default()
    });
    let outcome = build(
        vec!["a.feature", "b.feature"],
        folders.clone(),
        writer.clone(),
    );
    assert!(matches!(outcome, Err(BuildError::Input(_))));
    assert_eq!(
        *folders.done.lock().expect("lock"),
        ["clear site/v1.0.0", "mark site/v1.0.0"]
    );
    assert_eq!(
        *writer.written.lock().expect("lock"),
        ["site/v1.0.0/a.html"]
    );
}
