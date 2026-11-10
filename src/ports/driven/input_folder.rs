//! Port: what the build needs to see of the input folder, its listings and its files' text.

use std::io::Result as IoResult;
use std::path::Path;

use crate::entities::FolderEntry;

pub trait InputFolder: Send + Sync {
    fn entries(&self, folder: &Path) -> IoResult<Vec<FolderEntry>>;

    fn read_text(&self, path: &Path) -> IoResult<String>;
}
