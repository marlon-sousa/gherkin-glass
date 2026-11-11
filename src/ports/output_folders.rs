//! Port: an output folder, what is in it, and its marker.

use std::io::Result as IoResult;
use std::path::Path;

use crate::entities::{OutputContents, Record};

pub(crate) trait OutputFolders: Send + Sync {
    /// What is at `output`'s place: every file and folder below it, and what
    /// its marker records.
    fn contents(&self, output: &Path) -> IoResult<OutputContents>;

    /// Leaves `folder` existing and empty, creating it and the folders above when missing.
    fn clear(&self, folder: &Path) -> IoResult<()>;

    /// Removes what is at `path`, a file, a link or a folder with all in it,
    /// never following a link; nothing when there is nothing there.
    fn remove(&self, path: &Path) -> IoResult<()>;

    /// Writes `record` as the marker of `output`, creating the folder when missing.
    fn record(&self, output: &Path, record: &Record) -> IoResult<()>;
}
