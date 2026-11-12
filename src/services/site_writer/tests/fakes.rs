//! The site writer's fakes: an output folder held in memory, an input, a parser and a writer.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{ErrorKind, Result as IoResult};
use std::path::PathBuf;
use std::sync::Mutex;

use gherkin::{Feature, GherkinEnv};

use super::*;
use crate::entities::{OutputContents, PageOptions, RawFeature, Unnamed};
use crate::errors::BuildError;
use crate::errors::{InputErrorKind, ParseError};
use crate::fixtures::MemoryFolders;
use crate::ports::FeatureParser;
use crate::services::{Converter, InputService};

/// An output folder held in memory. Each look into it takes the next of
/// `looks`, and the last one again once they run out; `None` cannot be read.
/// Each operation named in `fails` fails, and every other is recorded.
#[derive(Default)]
pub(super) struct FakeFolders {
    pub(super) looks: Mutex<Vec<Option<OutputContents>>>,
    pub(super) done: Mutex<Vec<String>>,
    pub(super) recorded: Mutex<Option<Record>>,
    pub(super) fails: Vec<&'static str>,
}

impl FakeFolders {
    pub(super) fn looking(looks: Vec<Option<OutputContents>>) -> Arc<Self> {
        Arc::new(FakeFolders {
            looks: Mutex::new(looks),
            ..FakeFolders::default()
        })
    }

    pub(super) fn holding(contents: OutputContents) -> Arc<Self> {
        Self::looking(vec![Some(contents)])
    }

    fn note(&self, operation: &str, path: &Path) -> IoResult<()> {
        if self.fails.contains(&operation) {
            return Err(IoError::new(ErrorKind::PermissionDenied, "denied"));
        }
        self.done
            .lock()
            .expect("lock")
            .push(format!("{operation} {}", path.display()));
        Ok(())
    }

    pub(super) fn done(&self) -> Vec<String> {
        self.done.lock().expect("lock").clone()
    }
}

impl OutputFolders for FakeFolders {
    fn contents(&self, _output: &Path) -> IoResult<OutputContents> {
        let mut looks = self.looks.lock().expect("lock");
        let look = if looks.len() > 1 {
            looks.remove(0)
        } else {
            looks.first().cloned().flatten()
        };
        look.ok_or_else(|| IoError::new(ErrorKind::PermissionDenied, "denied"))
    }

    fn clear(&self, folder: &Path) -> IoResult<()> {
        self.note("clear", folder)
    }

    fn remove(&self, path: &Path) -> IoResult<()> {
        self.note("remove", path)
    }

    fn record(&self, output: &Path, record: &Record) -> IoResult<()> {
        self.note("record", output)?;
        *self.recorded.lock().expect("lock") = Some(record.clone());
        Ok(())
    }
}

pub(super) fn name(text: &str) -> VersionName {
    VersionName::new(text).expect("a version name")
}

pub(super) fn set(paths: &[&str]) -> BTreeSet<FilePath> {
    paths.iter().map(|path| path.to_string()).collect()
}

/// A folder holding `files` and `folders`, whose marker records `record`.
pub(super) fn folder(files: &[&str], folders: &[&str], record: Option<Record>) -> OutputContents {
    OutputContents::Folder {
        files: set(files),
        folders: set(folders),
        record,
    }
}

/// The output folder of the versions `main` and `v1.0.0`, as ghg wrote it.
pub(super) fn two_versions() -> OutputContents {
    let versions = BTreeMap::from([
        (name("main"), set(&["index.html"])),
        (name("v1.0.0"), set(&["old.html"])),
    ]);
    let record = Record::Versions {
        page: set(&["index.html"]),
        versions,
    };
    folder(
        &[
            "ghg-versions.json",
            "index.html",
            "main/index.html",
            "v1.0.0/old.html",
        ],
        &["main", "v1.0.0"],
        Some(record),
    )
}

/// Parses every file as a feature named after it, but fails `bad.feature`.
pub(super) struct FakeParser;

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
pub(super) struct FakeWriter {
    pub(super) written: Mutex<Vec<String>>,
    pub(super) fails: Option<&'static str>,
}

impl FileWriter for FakeWriter {
    fn write(&self, output: &Path, file: &str, _contents: &[u8]) -> IoResult<()> {
        if self.fails == Some(file) {
            return Err(IoError::other("disk full"));
        }
        let path = format!("{}/{file}", output.display());
        self.written.lock().expect("lock").push(path);
        Ok(())
    }

    fn site(&self, output: &Path) -> ServedSite {
        ServedSite::Folder(output.to_path_buf())
    }
}

/// Builds the features `files` into `site`, as the version `version` when
/// there is one, as the controller does: the folder is checked, the input
/// read and converted, and the site written.
pub(super) fn build(
    files: Vec<&'static str>,
    version: Option<&str>,
    folders: Arc<FakeFolders>,
    writer: Arc<FakeWriter>,
) -> Result<ServedSite, BuildError> {
    let folders_in = MemoryFolders::with_files_in(Path::new("in"), &files);
    let input = InputService::new(Arc::new(folders_in), Arc::new(FakeParser));
    let settings = Settings {
        input: PathBuf::from("in"),
        output: PathBuf::from("site"),
        destination: Destination::Folder,
        unnamed: Unnamed::Allowed,
        pages: PageOptions::default(),
        version: version.map(name),
        replace: true,
    };
    let site_writer = SiteWriter::new(folders, writer, Arc::new(FakeWriter::default()));
    site_writer.check(&settings)?;
    let features = input.read(&settings.input, settings.unnamed)?;
    Ok(site_writer.write(&settings, Converter.convert(&features, &settings))?)
}

pub(super) fn written(writer: &FakeWriter) -> Vec<String> {
    writer.written.lock().expect("lock").clone()
}
