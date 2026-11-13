//! An input held in memory that a test changes between builds, and the changes a burst tells of.

use std::collections::BTreeSet;
use std::io::{Error as IoError, ErrorKind, Result as IoResult};
use std::mem::take;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use gherkin::{Feature, GherkinEnv};

use crate::entities::{FolderEntry, OutputContents, RawFeature, Record};
use crate::errors::{InputErrorKind, ParseError};
use crate::ports::{ConfigReader, FeatureParser, InputFolders, OutputFolders};
use crate::test_support::MemoryFolders;

/// An input held in memory that a test changes between builds, read by the input collector and
/// parsed by the gherkin parser, with no configuration file and no output folder.
#[derive(Default)]
pub(super) struct ChangingInput {
    folders: Mutex<MemoryFolders>,
}

impl ChangingInput {
    pub(super) fn apply(&self, change: &Change) {
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
#[derive(Debug, Clone)]
pub(super) enum Change {
    /// A file made or changed, with its text, by its path.
    Write(&'static str, &'static str),
    /// A file removed, or a folder with everything in it, by its path.
    Remove(&'static str),
    /// A folder moved, with everything in it, by both its paths, merged into the folder at the
    /// second when there is one.
    Rename(&'static str, &'static str),
    /// A file made or changed in a folder, with its text, by the folder's path alone, as a
    /// watcher tells of a folder put in place of another.
    Replace(&'static str, &'static str, &'static str),
}

impl Change {
    /// The paths the burst names.
    pub(super) fn paths(&self) -> BTreeSet<PathBuf> {
        let paths = match self {
            Change::Write(path, _) | Change::Remove(path) | Change::Replace(path, ..) => {
                vec![path]
            }
            Change::Rename(from, to) => vec![from, to],
        };
        paths.into_iter().map(PathBuf::from).collect()
    }
}
