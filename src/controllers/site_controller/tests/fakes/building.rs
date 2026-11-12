//! The fakes of the services a build runs: the settings service, the gherkin parser, the
//! converter and the site writer.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use gherkin::{Feature, GherkinEnv};

use super::*;
use crate::entities::{PageOptions, ParsedFeature, SiteNode, SiteTree, SiteTreeBuilder, Unnamed};
use crate::policies::page_frame;

/// Gives the settings of a build from the input `options` name, or `features`, into `site`, or,
/// while `failing`, the errors of [`config_errors`].
#[derive(Default)]
pub(crate) struct FakeSettingsService {
    pub(super) calls: Arc<Calls>,
    pub(crate) failing: AtomicBool,
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
        Ok(Settings {
            input: options
                .input
                .clone()
                .unwrap_or_else(|| PathBuf::from("features")),
            output: PathBuf::from("site"),
            destination,
            unnamed: Unnamed::Refused,
            pages: PageOptions::default(),
            version: None,
            replace: true,
        })
    }
}

/// Finds one file, `a.feature`, in whatever input it is given, and parses it as a feature named
/// `a`, or, while `failing`, gives the errors of [`input_errors`].
#[derive(Default)]
pub(crate) struct FakeGherkinParser {
    pub(super) calls: Arc<Calls>,
    pub(crate) failing: AtomicBool,
}

impl GherkinParserApi for FakeGherkinParser {
    fn paths(&self, input: &Path) -> Vec<PathBuf> {
        self.calls
            .note(format!("gherkin_parser.paths {}", input.display()));
        vec![input.join("a.feature")]
    }

    fn parse(&self, input: &Path, unnamed: Unnamed) -> Result<Vec<ParsedFeature>, InputErrors> {
        self.calls.note(format!(
            "gherkin_parser.parse {} {unnamed:?}",
            input.display()
        ));
        if self.failing.load(Ordering::SeqCst) {
            return Err(input_errors());
        }
        let feature = Feature::parse("Feature: a\n", GherkinEnv::default()).expect("parses");
        Ok(vec![ParsedFeature::new(
            PathBuf::from("a.feature"),
            feature,
            "en".into(),
            BTreeMap::new(),
        )])
    }
}

/// Makes a tree with one asset for each feature, named after the feature's path.
pub(crate) struct FakeConverter {
    pub(super) calls: Arc<Calls>,
}

impl ConverterApi for FakeConverter {
    fn convert(&self, features: &[ParsedFeature], settings: &Settings) -> SiteTree {
        let tree = SiteTreeBuilder::new(page_frame(&settings.pages))
            .assets(features.iter().map(asset_of))
            .build();
        self.calls
            .note(format!("converter.convert {}", asset_names(&tree)));
        tree
    }
}

/// Writes nothing: a site into a folder is that folder, and one in memory holds one empty file
/// named after how many sites were written, `site 1` first. The check fails while
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
        tree: &SiteTree,
    ) -> Result<(BuiltSite, ServedSite), OutputError> {
        self.calls
            .note(format!("site_writer.write {}", asset_names(tree)));
        if self.write_failing.load(Ordering::SeqCst) {
            return Err(output_error());
        }
        let number = self.written.fetch_add(1, Ordering::SeqCst) + 1;
        let site = match settings.destination {
            Destination::Folder => ServedSite::Folder(settings.output.clone()),
            Destination::Memory => {
                ServedSite::Memory(BTreeMap::from([(format!("site {number}"), Vec::new())]))
            }
        };
        Ok((built_site(), site))
    }
}

/// The names of the assets among the files of `tree`, joined with spaces.
fn asset_names(tree: &SiteTree) -> String {
    let names: Vec<_> = tree.files().filter_map(asset_name).collect();
    names.join(" ")
}

/// The name of the asset that stands for `feature`.
fn asset_of(feature: &ParsedFeature) -> String {
    feature.path.display().to_string()
}

fn asset_name(file: &SiteNode) -> Option<&str> {
    match file {
        SiteNode::Asset(name) => Some(name),
        _ => None,
    }
}
