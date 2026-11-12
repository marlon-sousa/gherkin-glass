//! Test support: site trees of features written in the test, the fixtures under `tests/fixtures`, read as the command reads them, scratch
//! folders for tests that touch the disk, an input held in memory, output folders as ghg finds
//! them, and a server and a watcher for ghg serve.

mod output;
mod serving;

use std::collections::{BTreeMap, BTreeSet};
#[cfg(unix)]
use std::fs::{Permissions, read_to_string, set_permissions};
use std::fs::{copy, create_dir_all, read_dir, remove_dir_all};
use std::io::{Error as IoError, ErrorKind, Result as IoResult};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use gherkin::{Feature, GherkinEnv};
use regex_automata::meta::Regex;

use crate::adapters::{GherkinCrate, InputFolderReader};
use crate::entities::{
    EntryKind, FolderEntry, InputTree, PageOptions, ParsedFeature, SiteFeaturePage, SiteTree,
    TagLink, Unnamed,
};
use crate::errors::InputErrors;
use crate::policies::site_tree;
use crate::ports::{FeatureParser, InputFolders};
use crate::services::{GherkinParser, GherkinParserApi, InputCollector, InputCollectorApi};

pub(crate) use output::{folder, name, set, two_versions};
pub(crate) use serving::{FakeServer, FakeWatcher, changed};

/// A link from the tags that `pattern` matches whole, as the configuration gives it, for a
/// pattern without the `x` flag, whose trailing comment would swallow the anchors written here.
pub(crate) fn tag_link(pattern: &str, address: &str, text: &str) -> TagLink {
    TagLink {
        source: pattern.into(),
        pattern: Regex::new(&format!("^(?:{pattern})$")).expect("a regular expression"),
        address: address.into(),
        text: text.into(),
    }
}

/// The feature of `text`, at `path` below the input folder, written in English.
pub(crate) fn parsed_at(path: &str, text: &str) -> ParsedFeature {
    let feature = Feature::parse(text, GherkinEnv::default()).expect("parses");
    ParsedFeature::new(
        PathBuf::from(path),
        feature,
        "en".to_string(),
        BTreeMap::new(),
    )
}

/// The site tree of `features`, each a path below the input folder and its text, for a site
/// without versions, with the settings of every page `options`.
pub(crate) fn site_of(features: &[(&str, &str)], options: &PageOptions) -> SiteTree {
    let parsed: Vec<_> = features
        .iter()
        .map(|(path, text)| parsed_at(path, text))
        .collect();
    site_tree(&parsed, options, None, &BTreeSet::new())
}

/// Every feature page of `site`, folder by folder from the input folder in.
pub(crate) fn feature_pages(site: &SiteTree) -> Vec<SiteFeaturePage<'_>> {
    site.files()
        .filter_map(|file| file.feature_page())
        .collect()
}

/// The fixture folder `name`, under `tests/fixtures`.
pub(crate) fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// The input tree of the folder `input`, collected through `input_folders`, with its files
/// parsed through `feature_parser`, as a build from nothing makes it.
pub(crate) fn parsed_input(
    input_folders: Arc<dyn InputFolders>,
    feature_parser: Arc<dyn FeatureParser>,
    input: &Path,
    unnamed: Unnamed,
) -> InputTree {
    let mut input_tree = InputCollector::new(input_folders).collect(input);
    let files = input_tree.files_to_parse();
    input_tree.put_parsed(GherkinParser::new(feature_parser).parse(input, files, unnamed));
    input_tree
}

/// The features of `input_tree`, or every error it holds.
pub(crate) fn features_of(input_tree: &InputTree) -> Result<Vec<ParsedFeature>, InputErrors> {
    match input_tree.input_errors() {
        Some(errors) => Err(errors),
        None => Ok(input_tree.features().cloned().collect()),
    }
}

/// The features of the folder `input`, or the errors found, collected and parsed over the real
/// file system, as the command reads them.
pub(crate) fn read_input(input: &Path) -> Result<Vec<ParsedFeature>, InputErrors> {
    let input_folders = Arc::new(InputFolderReader);
    let feature_parser = Arc::new(GherkinCrate);
    features_of(&parsed_input(
        input_folders,
        feature_parser,
        input,
        Unnamed::Allowed,
    ))
}

/// The features of the folder `input`, which reads with no error.
pub(crate) fn read_features(input: &Path) -> Vec<ParsedFeature> {
    read_input(input).unwrap_or_else(|errors| panic!("the input is read:\n{errors}"))
}

/// A folder of its own, emptied first, under the crate's `target/unit-tests`. Each module's
/// `name`s start with its own folder, such as `input/`, so that no two tests share one.
pub(crate) fn scratch(name: &str) -> PathBuf {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/unit-tests")
        .join(name);
    let _ = remove_dir_all(&folder);
    create_dir_all(&folder).expect("the scratch folder is made");
    folder
}

/// Makes `path` unreadable, and says whether it worked: it does not for root.
#[cfg(unix)]
pub(crate) fn lock(path: &Path) -> bool {
    set_permissions(path, Permissions::from_mode(0o000)).expect("the mode is set");
    let locked = read_dir(path).is_err() && read_to_string(path).is_err();
    if !locked {
        eprintln!(
            "skipped: {} can still be read, as it can by root",
            path.display()
        );
    }
    locked
}

#[cfg(unix)]
pub(crate) fn unlock(path: &Path, mode: u32) {
    set_permissions(path, Permissions::from_mode(mode)).expect("the mode is set");
}

/// Copies the folder `from`, with everything in it, to `to`.
pub(crate) fn copy_folder(from: &Path, to: &Path) {
    create_dir_all(to).expect("the folder is made");
    for entry in read_dir(from).expect("the folder is listed") {
        let entry = entry.expect("an entry of the folder");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("the entry has a type").is_dir() {
            copy_folder(&entry.path(), &target);
        } else {
            copy(entry.path(), &target).expect("the file is copied");
        }
    }
}

/// An input held in memory: its files, each with its text, and the folders that cannot be
/// listed. A folder is listed with the files and folders under it; one with nothing under it is
/// empty.
#[derive(Debug, Default)]
pub(crate) struct MemoryFolders {
    pub(crate) files: BTreeMap<PathBuf, String>,
    pub(crate) unlistable: BTreeSet<PathBuf>,
}

impl MemoryFolders {
    /// The files at `paths`, each with no text.
    pub(crate) fn with_files<P: Into<PathBuf>>(paths: impl IntoIterator<Item = P>) -> Self {
        MemoryFolders {
            files: paths.into_iter().map(empty_file).collect(),
            unlistable: BTreeSet::new(),
        }
    }

    /// The files at `paths` in the folder `input`, each with no text.
    pub(crate) fn with_files_in(input: &Path, paths: &[&str]) -> Self {
        MemoryFolders::with_files(paths.iter().map(|path| input.join(path)))
    }
}

impl InputFolders for MemoryFolders {
    fn list(&self, folder: &Path) -> IoResult<Vec<FolderEntry>> {
        if self.unlistable.contains(folder) {
            return Err(IoError::new(
                ErrorKind::PermissionDenied,
                "permission denied",
            ));
        }
        let mut entries: Vec<_> = self
            .files
            .keys()
            .filter_map(|path| entry_under(folder, path))
            .collect();
        // The files are sorted, so the entries of one folder below are side by side.
        entries.dedup();
        Ok(entries)
    }

    fn read(&self, file: &Path) -> IoResult<String> {
        self.files
            .get(file)
            .cloned()
            .ok_or_else(|| IoError::new(ErrorKind::NotFound, "not found"))
    }
}

/// The entry of `folder` that `path` is or is under, if it is under `folder`.
fn entry_under(folder: &Path, path: &Path) -> Option<FolderEntry> {
    let mut parts = path.strip_prefix(folder).ok()?.components();
    let path = folder.join(parts.next()?);
    let kind = match parts.next() {
        Some(_) => EntryKind::Folder,
        None => EntryKind::File,
    };
    Some(FolderEntry { path, kind })
}

fn empty_file<P: Into<PathBuf>>(path: P) -> (PathBuf, String) {
    (path.into(), String::new())
}
