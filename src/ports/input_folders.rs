//! Port: listing a folder of the input, reading a file in it, and making a path absolute from the current folder.

use std::io::Result as IoResult;
use std::path::{Path, PathBuf};

use crate::entities::FolderEntry;

pub(crate) trait InputFolders: Send + Sync {
    /// The entries of `folder`, in no order.
    fn list(&self, folder: &Path) -> IoResult<Vec<FolderEntry>>;

    fn read(&self, file: &Path) -> IoResult<String>;

    /// `path` from the current folder when it is relative, as written otherwise, without
    /// looking at what is there.
    fn absolute(&self, path: &Path) -> IoResult<PathBuf>;
}
