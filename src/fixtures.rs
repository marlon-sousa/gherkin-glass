//! Test support: the fixtures under `tests/fixtures`, read as the command reads them, scratch
//! folders for tests that touch the disk, an input held in memory, and a server and a watcher
//! for ghg serve.

mod serving;

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{copy, create_dir_all, read_dir, remove_dir_all};
use std::io::{Error as IoError, ErrorKind, Result as IoResult};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::adapters::{GherkinCrate, InputFolderReader};
use crate::entities::{EntryKind, FolderEntry, ParsedFeature, Unnamed};
use crate::errors::InputErrors;
use crate::ports::InputFolders;
use crate::services::InputService;

pub(crate) use serving::{FakeServer, FakeWatcher, changed};

/// The fixture folder `name`, under `tests/fixtures`.
pub(crate) fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// The features of the folder `input`, or the errors found, read by the input service over the
/// real file system, as the command reads them.
pub(crate) fn read_input(input: &Path) -> Result<Vec<ParsedFeature>, InputErrors> {
    InputService::new(Arc::new(InputFolderReader), Arc::new(GherkinCrate))
        .read(input, Unnamed::Allowed)
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
