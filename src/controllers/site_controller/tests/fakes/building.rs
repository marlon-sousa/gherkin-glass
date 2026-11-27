//! The fakes of the services a build runs: the settings service, the input collector, the
//! gherkin parser, the converter and the site writer.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use gherkin::{Feature, GherkinEnv};

use super::*;
use crate::entities::{
    BuildTarget, CollectedChanges, FileToParse, IgnoreRules, InputTree, InputTreeBuilder,
    KeptInOutput, PageOptions, ParsedFeature, ParsedFile, PathList, Settings, SiteStore, Unnamed,
};

/// Gives the settings of a build from the input the configuration names, once a test sets it, or
/// else from the one `options` name, or `features`, into `site`, or, while `failing`, the errors
/// of [`config_errors`]. Once it has given them, the configuration names `next_configured_input`,
/// when a test sets it, as though the configuration file was saved meanwhile, and fails from then
/// on when a test sets `failing_next`. Its front page's template is `front_page`.
#[derive(Default)]
pub(crate) struct FakeSettingsService {
    pub(super) calls: Arc<Calls>,
    pub(crate) failing: AtomicBool,
    pub(crate) failing_next: AtomicBool,
    pub(crate) configured_input: Mutex<Option<PathBuf>>,
    pub(crate) next_configured_input: Mutex<Option<PathBuf>>,
    pub(crate) front_page: Mutex<Option<PathBuf>>,
}

impl SettingsApi for FakeSettingsService {
    fn config_file(&self, options: &BuildOptions) -> PathBuf {
        self.calls.note("settings.config_file".into());
        options
            .config
            .clone()
            .unwrap_or_else(|| PathBuf::from("ghg.toml"))
    }

    fn load(
        &self,
        options: &BuildOptions,
        destination: Destination,
    ) -> Result<Settings, ConfigErrors> {
        self.calls.note(format!("settings.load {destination:?}"));
        if self.failing.load(Ordering::SeqCst) {
            return Err(config_errors());
        }
        if self.failing_next.swap(false, Ordering::SeqCst) {
            self.failing.store(true, Ordering::SeqCst);
        }
        let mut configured_input = self
            .configured_input
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let configured = configured_input.clone();
        if let Some(next) = self
            .next_configured_input
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()
        {
            *configured_input = Some(next);
        }
        Ok(Settings {
            input: configured
                .or_else(|| options.input.clone())
                .unwrap_or_else(|| PathBuf::from("features")),
            output: PathBuf::from("site"),
            destination,
            unnamed: Unnamed::Refused,
            pages: PageOptions::default(),
            target: options.target.clone(),
            replace: true,
            front_page: self
                .front_page
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .clone(),
            only: None,
            keep_in_output: KeptInOutput::default(),
        })
    }
}

/// Finds one file, `a.feature`, with nothing parsed, in whatever input it is given, from nothing
/// or from the last tree, and says that every change calls for a rebuild, unless
/// `nothing_read_changes`; from nothing, the errors of [`config_errors`] instead, as for a list
/// of `--only` that cannot be read, while `list_failing`, and no file, while `no_file`, beside
/// the input folder's error that it does not exist, while `input_missing` too.
#[derive(Default)]
pub(crate) struct FakeInputCollector {
    pub(super) calls: Arc<Calls>,
    pub(crate) nothing_read_changes: AtomicBool,
    pub(crate) list_failing: AtomicBool,
    pub(crate) no_file: AtomicBool,
    pub(crate) input_missing: AtomicBool,
}

impl InputCollectorApi for FakeInputCollector {
    fn collect(&self, input: &Path, _: Option<&PathList>) -> Result<InputTree, ConfigErrors> {
        self.calls
            .note(format!("input_collector.collect {}", input.display()));
        if self.list_failing.load(Ordering::SeqCst) {
            return Err(config_errors());
        }
        if !self.no_file.load(Ordering::SeqCst) {
            return Ok(one_file_in(input));
        }
        let builder = InputTreeBuilder::listed(input.to_path_buf(), IgnoreRules::default());
        if !self.input_missing.load(Ordering::SeqCst) {
            return Ok(builder.build());
        }
        let missing = InputError::new(input, input, None, InputErrorKind::InputMissing);
        Ok(InputTree::unlisted_input(input.to_path_buf(), missing))
    }

    fn collect_changes(&self, last: &InputTree, paths: &BTreeSet<PathBuf>) -> CollectedChanges {
        // Read before the call is noted, since a test changes it once it sees the call.
        let calls_for_rebuild = !self.nothing_read_changes.load(Ordering::SeqCst);
        let paths: Vec<_> = paths
            .iter()
            .map(|path| path.display().to_string())
            .collect();
        self.calls.note(format!(
            "input_collector.collect_changes {} {}",
            last.input_folder().display(),
            paths.join(" ")
        ));
        CollectedChanges {
            input_tree: one_file_in(last.input_folder()),
            calls_for_rebuild,
        }
    }
}

/// The tree of `input` with one file, `a.feature`, with nothing parsed.
fn one_file_in(input: &Path) -> InputTree {
    let mut builder = InputTreeBuilder::listed(input.to_path_buf(), IgnoreRules::default());
    let root = builder.root();
    builder.add_allowed_file(root, input.join("a.feature"));
    builder.build()
}

/// Parses each file as a feature named after its name, or, while `failing`, gives the errors of
/// [`input_errors`] for each.
#[derive(Default)]
pub(crate) struct FakeGherkinParser {
    pub(super) calls: Arc<Calls>,
    pub(crate) failing: AtomicBool,
}

impl GherkinParserApi for FakeGherkinParser {
    fn parse(&self, input: &Path, files: Vec<FileToParse>, unnamed: Unnamed) -> Vec<ParsedFile> {
        let names: Vec<_> = files.iter().map(|file| file_name(input, file)).collect();
        self.calls.note(format!(
            "gherkin_parser.parse {} {unnamed:?}",
            names.join(" ")
        ));
        let failing = self.failing.load(Ordering::SeqCst);
        files
            .into_iter()
            .map(|file| parsed(input, file, failing))
            .collect()
    }
}

fn file_name(input: &Path, file: &FileToParse) -> String {
    let relative = file.path.strip_prefix(input).expect("under the input");
    relative.display().to_string()
}

fn parsed(input: &Path, file: FileToParse, failing: bool) -> ParsedFile {
    let result = if failing {
        Err(input_errors().as_slice().to_vec())
    } else {
        let name = file_name(input, &file);
        let feature =
            Feature::parse(format!("Feature: {name}\n"), GherkinEnv::default()).expect("parses");
        Ok(ParsedFeature::new(
            PathBuf::from(name),
            feature,
            "en".into(),
            BTreeMap::new(),
        ))
    };
    ParsedFile { file, result }
}

/// Makes an empty store, and notes the paths of the features it is given.
pub(crate) struct FakeConverter {
    pub(super) calls: Arc<Calls>,
}

impl ConverterApi for FakeConverter {
    fn convert(&self, input_tree: &InputTree, _: &Settings) -> SiteStore {
        let paths: Vec<String> = input_tree
            .features()
            .map(|parsed| parsed.path.display().to_string())
            .collect();
        self.calls
            .note(format!("converter.convert {}", paths.join(" ")));
        SiteStore::default()
    }
}

/// Writes nothing: a site into a folder is that folder, and one in memory holds one empty file
/// named after how many sites were written, `site 1` first. It finds the version `v1.0.0` in
/// the output folder, and nothing else, so removes only that. The check fails while
/// `check_failing`, and the writing while `write_failing`, each with [`output_error`].
#[derive(Default)]
pub(crate) struct FakeSiteWriter {
    pub(super) calls: Arc<Calls>,
    pub(crate) check_failing: AtomicBool,
    pub(crate) write_failing: AtomicBool,
    pub(super) written: AtomicUsize,
}

impl SiteWriterApi for FakeSiteWriter {
    fn check(&self, settings: &Settings) -> Result<(), OutputError> {
        self.calls
            .note(format!("site_writer.check {:?}", settings.destination));
        if self.check_failing.load(Ordering::SeqCst) {
            return Err(output_error());
        }
        Ok(())
    }

    fn write(
        &self,
        settings: &Settings,
        _: &SiteStore,
    ) -> Result<(ServedSite, BuiltSite), BuildError> {
        self.calls.note("site_writer.write".into());
        if self.write_failing.load(Ordering::SeqCst) {
            return Err(BuildError::Output(output_error()));
        }
        let number = self.written.fetch_add(1, Ordering::SeqCst) + 1;
        let site = match settings.destination {
            Destination::Folder => ServedSite::Folder(settings.output.clone()),
            Destination::Memory => {
                ServedSite::Memory(BTreeMap::from([(format!("site {number}"), Arc::from([]))]))
            }
        };
        Ok((site, built_site()))
    }

    fn remove(&self, settings: &Settings) -> Result<bool, BuildError> {
        self.calls
            .note(format!("site_writer.remove {:?}", settings.target));
        if self.write_failing.load(Ordering::SeqCst) {
            return Err(BuildError::Output(output_error()));
        }
        Ok(settings.target == BuildTarget::Version(crate::test_support::name("v1.0.0")))
    }
}
