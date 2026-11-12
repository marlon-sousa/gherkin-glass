//! Port: listing a folder of the input and reading a file in it.

use std::io::Result as IoResult;
use std::path::Path;

use crate::entities::FolderEntry;

pub(crate) trait InputFolders: Send + Sync {
    /// The entries of `folder`, in no order.
    fn list(&self, folder: &Path) -> IoResult<Vec<FolderEntry>>;

    fn read(&self, file: &Path) -> IoResult<String>;
}
