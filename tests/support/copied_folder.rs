//! Test support: a copy of a committed folder, for a test that changes it. The tests in `tests/` bring it in with `#[path]`.

use std::fs::{copy, create_dir_all, read_dir};
use std::path::Path;

/// Copies everything below `from` into `to`, made as needed.
pub(crate) fn copy_folder(from: &Path, to: &Path) {
    create_dir_all(to).expect("the folder is made");
    for entry in read_dir(from).expect("the folder is listed") {
        let entry = entry.expect("the folder is listed");
        let path = entry.path();
        if path.is_dir() {
            copy_folder(&path, &to.join(entry.file_name()));
        } else {
            copy(&path, to.join(entry.file_name())).expect("the file is copied");
        }
    }
}
