//! Service: writing a build into its output folder, with or without versions, and the versions it holds.

use std::io::Error as IoError;
use std::iter::once;
use std::path::Path;
use std::sync::Arc;

use crate::entities::{
    BuildFailure, BuildFolder, FolderState, Marker, OutputProblem, OutputProblemKind, Problem,
    ProblemKind, Problems, Settings, VersionList, VersionName,
};
use crate::policies::{ASSETS, VERSIONS_PAGE, order_versions, render_versions_page};
use crate::ports::OutputFolders;
use crate::services::{SiteService, assets};

#[derive(Clone)]
pub struct OutputService {
    folders: Arc<dyn OutputFolders>,
}

impl OutputService {
    pub fn new(folders: Arc<dyn OutputFolders>) -> Self {
        OutputService { folders }
    }

    /// Builds the site into `settings.output`, as the version its settings
    /// name, or without versions when they name none.
    pub(crate) fn build(
        &self,
        site: &SiteService,
        settings: &Settings,
    ) -> Result<(), BuildFailure> {
        match &settings.version {
            Some(version) => self.build_version(site, settings, version),
            None => self.build_site(site, settings),
        }
    }

    /// Builds a site without versions into `settings.output`, as
    /// docs/BEHAVIOUR.md, rule 38, says. The output folder is emptied only
    /// once the input has been read without a problem.
    pub(crate) fn build_site(
        &self,
        site: &SiteService,
        settings: &Settings,
    ) -> Result<(), BuildFailure> {
        let output = &settings.output;
        let folder = self.check_site(output, settings.replace)?;
        let files = site.files(settings)?;
        self.make_room(output, &folder)?;
        site.write(output, files)?;
        Ok(())
    }

    /// Builds the version `version` of the site into its folder in
    /// `settings.output`, then writes the versions page, as docs/BEHAVIOUR.md,
    /// "Versions", says. The version's folder is emptied only once the input
    /// has been read without a problem.
    pub(crate) fn build_version(
        &self,
        site: &SiteService,
        settings: &Settings,
        version: &VersionName,
    ) -> Result<(), BuildFailure> {
        let output = &settings.output;
        let folder = self.check(output, version, settings.replace)?;
        let files = site.files(settings)?;
        self.make_room(output, &folder)?;
        site.write(folder.path(), files)?;
        let page = render_versions_page(&self.versions(output)?, &settings.pages);
        // What is at the page's place and its assets' is the versions page's own.
        for place in [VERSIONS_PAGE, ASSETS] {
            self.folders.remove(&output.join(place)).map_err(|error| {
                Problems::new(vec![Problem::of_site(
                    output,
                    place,
                    ProblemKind::CannotBeWritten {
                        reason: reason(&error),
                    },
                )])
            })?;
        }
        let root = once((VERSIONS_PAGE.to_string(), page.into_bytes())).chain(assets());
        site.write(output, root)?;
        Ok(())
    }

    /// The output folder a site without versions is written into, once
    /// nothing stops it, as docs/BEHAVIOUR.md, rule 38, says. Nothing is written.
    pub(crate) fn check_site(
        &self,
        output: &Path,
        replace: bool,
    ) -> Result<BuildFolder, OutputProblem> {
        let problem = |kind| OutputProblem {
            output: output.to_path_buf(),
            version: None,
            kind,
        };
        let versions = self.folders.marked(output).map_err(|error| {
            problem(OutputProblemKind::CannotBeRead {
                reason: reason(&error),
            })
        })?;
        if !versions.is_empty() {
            return Err(problem(OutputProblemKind::HoldsVersions));
        }
        match self.folders.state(output, &Marker::Site) {
            Ok(FolderState::Missing | FolderState::Empty) => {
                Ok(BuildFolder::new(output, Marker::Site))
            }
            Ok(FolderState::Marked) if replace => Ok(BuildFolder::new(output, Marker::Site)),
            Ok(FolderState::Marked) => Err(problem(OutputProblemKind::AlreadyBuilt)),
            Ok(FolderState::Other | FolderState::OtherCapitals) => {
                Err(problem(OutputProblemKind::NotASite))
            }
            Err(error) => Err(problem(OutputProblemKind::CannotBeRead {
                reason: reason(&error),
            })),
        }
    }

    /// The folder the version `name` is written into, once nothing stops it,
    /// as docs/BEHAVIOUR.md, rules 123 and 124, say. Nothing is written.
    pub fn check(
        &self,
        output: &Path,
        name: &VersionName,
        replace: bool,
    ) -> Result<BuildFolder, OutputProblem> {
        let marker = Marker::Version(name.clone());
        let folder = BuildFolder::new(output, marker.clone());
        let problem = |version: Option<&VersionName>, kind| OutputProblem {
            output: output.to_path_buf(),
            version: version.cloned(),
            kind,
        };
        match self.folders.state(output, &Marker::Site) {
            Ok(FolderState::Marked) => return Err(problem(None, OutputProblemKind::HoldsSite)),
            Ok(_) => {}
            Err(error) => {
                return Err(problem(
                    None,
                    OutputProblemKind::CannotBeRead {
                        reason: reason(&error),
                    },
                ));
            }
        }
        let problem = |kind| problem(Some(name), kind);
        match self.folders.state(folder.path(), &marker) {
            Ok(FolderState::Missing | FolderState::Empty) => Ok(folder),
            Ok(FolderState::Marked) if replace => Ok(folder),
            Ok(FolderState::Marked) => Err(problem(OutputProblemKind::AlreadyBuilt)),
            Ok(FolderState::Other) => Err(problem(OutputProblemKind::NotAVersion)),
            Ok(FolderState::OtherCapitals) => Err(problem(OutputProblemKind::OtherCapitals)),
            Err(error) => Err(problem(OutputProblemKind::CannotBeRead {
                reason: reason(&error),
            })),
        }
    }

    /// Empties the folder a check gave, and marks it, ready for the site to
    /// be written into it. `output` is the one given to the check.
    pub fn make_room(&self, output: &Path, folder: &BuildFolder) -> Result<(), OutputProblem> {
        let version = match folder.marker() {
            Marker::Site => None,
            Marker::Version(name) => Some(name.clone()),
        };
        let problem = |kind| OutputProblem {
            output: output.to_path_buf(),
            version: version.clone(),
            kind,
        };
        // What was checked may have changed while the input was read, and
        // nothing unmarked is ever emptied.
        match self.folders.state(folder.path(), folder.marker()) {
            Ok(FolderState::Missing | FolderState::Empty | FolderState::Marked) => {}
            Ok(FolderState::OtherCapitals) => {
                return Err(problem(OutputProblemKind::OtherCapitals));
            }
            Ok(FolderState::Other) => {
                return Err(problem(match version {
                    Some(_) => OutputProblemKind::NotAVersion,
                    None => OutputProblemKind::NotASite,
                }));
            }
            Err(error) => {
                return Err(problem(OutputProblemKind::CannotBeRead {
                    reason: reason(&error),
                }));
            }
        }
        self.folders
            .clear(folder.path())
            .and_then(|()| self.folders.mark(folder.path(), folder.marker()))
            .map_err(|error| OutputProblem {
                output: output.to_path_buf(),
                version,
                kind: OutputProblemKind::CannotBeWritten {
                    reason: reason(&error),
                },
            })
    }

    /// Every version `output` holds, in the order of the versions page.
    pub fn versions(&self, output: &Path) -> Result<VersionList, OutputProblem> {
        self.folders
            .marked(output)
            .map(order_versions)
            .map_err(|error| OutputProblem {
                output: output.to_path_buf(),
                version: None,
                kind: OutputProblemKind::CannotBeListed {
                    reason: reason(&error),
                },
            })
    }
}

fn reason(error: &IoError) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::io::{ErrorKind, Result as IoResult};
    use std::path::PathBuf;
    use std::sync::Mutex;

    use gherkin::{Feature, GherkinEnv};

    use super::*;
    use crate::entities::{CollectedInput, PageOptions, ParseFailure, RawFeature, Unnamed};
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

    fn check(
        states: &[(&str, Option<FolderState>)],
        replace: bool,
    ) -> Result<PathBuf, OutputProblem> {
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
            Err(OutputProblem {
                output: PathBuf::from("site"),
                version: Some(name("v1")),
                kind: OutputProblemKind::AlreadyBuilt,
            })
        );
    }

    #[test]
    fn a_folder_that_is_not_the_version_is_never_replaced() {
        for (state, kind) in [
            (FolderState::Other, OutputProblemKind::NotAVersion),
            (FolderState::OtherCapitals, OutputProblemKind::OtherCapitals),
        ] {
            for replace in [true, false] {
                assert_eq!(
                    check(&[("site/v1", Some(state))], replace).map_err(|problem| problem.kind),
                    Err(kind.clone())
                );
            }
        }
        let unreadable = [("site/v1", None)];
        assert!(matches!(
            check(&unreadable, true).map_err(|problem| problem.kind),
            Err(OutputProblemKind::CannotBeRead { .. })
        ));
    }

    #[test]
    fn a_folder_that_cannot_be_emptied_or_listed_is_reported() {
        let folder = BuildFolder::new(Path::new("site"), Marker::Version(name("v1")));
        let service = OutputService::new(Arc::new(FakeFolders::holding(&[], vec!["clear"])));
        assert!(matches!(
            service.make_room(Path::new("site"), &folder),
            Err(OutputProblem {
                version: Some(_),
                kind: OutputProblemKind::CannotBeWritten { .. },
                ..
            })
        ));
        let service = OutputService::new(Arc::new(FakeFolders::holding(&[], vec!["list"])));
        assert!(matches!(
            service.versions(Path::new("site")),
            Err(OutputProblem {
                version: None,
                kind: OutputProblemKind::CannotBeListed { .. },
                ..
            })
        ));
    }

    struct FakeCollector(Vec<&'static str>);

    impl InputCollector for FakeCollector {
        fn collect(&self, input: &Path) -> CollectedInput {
            CollectedInput {
                paths: self.0.iter().map(|path| input.join(path)).collect(),
                problems: Vec::new(),
            }
        }
    }

    /// Parses every file as a feature named after it, but fails `bad.feature`.
    struct FakeParser;

    impl FeatureParser for FakeParser {
        fn parse(&self, path: &Path) -> Result<RawFeature, ParseFailure> {
            if path.ends_with("bad.feature") {
                return Err(ParseFailure {
                    line: Some(1),
                    kind: ProblemKind::Syntax,
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
    ) -> Result<(), BuildFailure> {
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
    ) -> Result<(), BuildFailure> {
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
    fn input_with_a_problem_leaves_an_earlier_site_as_it_was() {
        let folders = FakeFolders::with(&[("site", Some(FolderState::Marked))]);
        let writer = Arc::new(FakeWriter::default());
        let failure = build_site(vec!["bad.feature"], folders.clone(), writer.clone());
        assert!(matches!(failure, Err(BuildFailure::Problems(_))));
        assert_eq!(*folders.done.lock().expect("lock"), ["list site"]);
        assert!(writer.written.lock().expect("lock").is_empty());
    }

    #[test]
    fn a_page_of_a_site_that_cannot_be_written_is_reported_with_its_path() {
        let writer = Arc::new(FakeWriter {
            fails: Some("b.html"),
            ..FakeWriter::default()
        });
        let failure = build_site(
            vec!["a.feature", "b.feature"],
            FakeFolders::with(&[]),
            writer,
        );
        match failure {
            Err(BuildFailure::Problems(problems)) => assert_eq!(
                problems.to_string(),
                "site/b.html: cannot be written: disk full\n"
            ),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn an_output_folder_that_cannot_be_looked_into_is_reported_as_such() {
        let folders = FakeFolders::holding(&[], vec!["list"]);
        let problem = OutputService::new(Arc::new(folders))
            .check_site(Path::new("site"), true)
            .expect_err("unreadable");
        assert!(matches!(
            problem.kind,
            OutputProblemKind::CannotBeRead { .. }
        ));
    }

    #[test]
    fn a_folder_that_changed_since_its_check_is_not_emptied() {
        let folders = FakeFolders::with(&[("site", Some(FolderState::Other))]);
        let service = OutputService::new(folders.clone());
        let folder = BuildFolder::new(Path::new("site"), Marker::Site);
        let problem = service
            .make_room(Path::new("site"), &folder)
            .expect_err("changed");
        assert_eq!(problem.kind, OutputProblemKind::NotASite);
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
    fn input_with_a_problem_empties_nothing_and_writes_nothing() {
        let folders = FakeFolders::with(&[("site/v1.0.0", Some(FolderState::Marked))]);
        let writer = Arc::new(FakeWriter::default());
        let failure = build(vec!["bad.feature"], folders.clone(), writer.clone());
        assert!(matches!(failure, Err(BuildFailure::Problems(_))));
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
        let failure = build(
            vec!["a.feature", "b.feature"],
            folders.clone(),
            writer.clone(),
        );
        assert!(matches!(failure, Err(BuildFailure::Problems(_))));
        assert_eq!(
            *folders.done.lock().expect("lock"),
            ["clear site/v1.0.0", "mark site/v1.0.0"]
        );
        assert_eq!(
            *writer.written.lock().expect("lock"),
            ["site/v1.0.0/a.html"]
        );
    }
}
