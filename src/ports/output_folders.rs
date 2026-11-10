//! Port: an output folder, the folders of its versions, and their markers.

use std::io::Result as IoResult;
use std::path::Path;

use crate::entities::{FolderState, Marker, VersionName};

pub trait OutputFolders: Send + Sync {
    /// What is at `folder`, told apart by whether it holds the marker of
    /// `marker`'s kind, whatever the version it names.
    fn state(&self, folder: &Path, marker: &Marker) -> IoResult<FolderState>;

    /// Leaves `folder` existing and empty, creating it and the folders above when missing.
    fn clear(&self, folder: &Path) -> IoResult<()>;

    /// Removes what is at `path`, a file, a link or a folder with all in it,
    /// never following a link; nothing when there is nothing there.
    fn remove(&self, path: &Path) -> IoResult<()>;

    /// Writes the marker that says what `folder` is.
    fn mark(&self, folder: &Path, marker: &Marker) -> IoResult<()>;

    /// The folders directly inside `output` that hold a version's marker, in
    /// no particular order; none when `output` does not exist.
    fn marked(&self, output: &Path) -> IoResult<Vec<VersionName>>;
}
