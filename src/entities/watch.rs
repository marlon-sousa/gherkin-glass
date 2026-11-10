//! Entity: what ghg serve watches for changes, and the changes it is told of.

use std::any::Any;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Watched {
    /// A folder and everything in it.
    Folder(PathBuf),
    /// One file, whether it exists yet or not. A path with no file name,
    /// such as `..`, is watched as a folder, without its subfolders.
    File(PathBuf),
}

/// One message for each burst of changes, until the watching ends.
pub struct Changes {
    pub bursts: Receiver<()>,
    /// Whatever keeps the watching going; dropping it ends it.
    pub watching: Box<dyn Any + Send>,
}

/// Watching that cannot start: the path it stopped at, and the reason in
/// the operating system's or the `notify` crate's words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchFailure {
    pub path: PathBuf,
    pub reason: String,
}
