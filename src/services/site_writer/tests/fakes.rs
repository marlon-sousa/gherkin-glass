//! The site writer's fakes: an output folder held in memory, a parser and a writer.

use std::io::{ErrorKind, Result as IoResult};
use std::path::PathBuf;
use std::sync::Mutex;

use gherkin::{Feature, GherkinEnv};

use super::*;
use crate::entities::{OutputContents, PageOptions, RawFeature, Unnamed};
use crate::errors::{BuildError, ParseError};
pub(super) use crate::fixtures::{MemoryFolders, folder, name, set, two_versions};
use crate::ports::FeatureParser;
use crate::services::{Converter, ConverterApi, GherkinParser, GherkinParserApi};

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

/// Parses every file as a feature named after it.
pub(super) struct FakeParser;

impl FeatureParser for FakeParser {
    fn parse(&self, path: &Path) -> Result<RawFeature, ParseError> {
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

/// Has the site writer check `site` and write into it the site of the
/// features `files`, as the version `version` when there is one.
pub(super) fn build(
    files: Vec<&'static str>,
    version: Option<&str>,
    folders: Arc<FakeFolders>,
    writer: Arc<FakeWriter>,
) -> Result<ServedSite, BuildError> {
    let folders_in = MemoryFolders::with_files_in(Path::new("in"), &files);
    let input = GherkinParser::new(Arc::new(folders_in), Arc::new(FakeParser));
    let settings = settings(Destination::Folder, version);
    let site_writer = SiteWriter::new(folders, writer, Arc::new(FakeWriter::default()));
    let features = input
        .parse(&settings.input, settings.unnamed)
        .expect("features");
    site_writer.check(&settings)?;
    let (_, served) = site_writer.write(&settings, &Converter.convert(&features, &settings))?;
    Ok(served)
}

/// The settings of a build from `in` into `site`, as the version `version`
/// when there is one.
pub(super) fn settings(destination: Destination, version: Option<&str>) -> Settings {
    Settings {
        input: PathBuf::from("in"),
        output: PathBuf::from("site"),
        destination,
        unnamed: Unnamed::Allowed,
        pages: PageOptions::default(),
        version: version.map(name),
        replace: true,
    }
}

pub(super) fn written(writer: &FakeWriter) -> Vec<String> {
    writer.written.lock().expect("lock").clone()
}
