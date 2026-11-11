//! Test support: the fixtures under `tests/fixtures`, read as the command reads them, and
//! scratch folders for tests that touch the disk.

use std::fs::{copy, create_dir_all, read_dir, remove_dir_all};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::adapters::{GherkinParser, InputWalker};
use crate::entities::{ParsedFeature, Problems, Unnamed};
use crate::services::InputService;

/// The fixture folder `name`, under `tests/fixtures`.
pub(crate) fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// The features of the folder `input`, or the problems found, read by the input service over the
/// real file system, as the command reads them.
pub(crate) fn read_input(input: &Path) -> Result<Vec<ParsedFeature>, Problems> {
    InputService::new(Arc::new(InputWalker), Arc::new(GherkinParser)).read(input, Unnamed::Allowed)
}

/// The features of the folder `input`, which reads with no problem.
pub(crate) fn read_features(input: &Path) -> Vec<ParsedFeature> {
    read_input(input).unwrap_or_else(|problems| panic!("the input is read:\n{problems}"))
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
