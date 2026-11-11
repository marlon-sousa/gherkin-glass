//! Entity: what ghg serve watches for changes, and the changes it is told of.

use std::any::Any;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Watched {
    /// A folder and everything in it.
    Folder(PathBuf),
    /// One file, whether it exists yet or not. A path with no file name,
    /// such as `..`, is watched as a folder, without its subfolders.
    File(PathBuf),
}

/// What one burst of changes, such as an editor's when it saves, was.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Burst {
    /// The paths that changed, each written from the path watched that
    /// holds it, as that was given: `features/a.feature` under
    /// `Watched::Folder("features")`, or the file itself.
    Changed(Vec<PathBuf>),
    /// Changes may have been missed, so which ones is not known.
    Missed,
}

/// One message for each burst of changes, until the watching ends.
pub(crate) struct Changes {
    pub(crate) bursts: Receiver<Burst>,
    /// Whatever keeps the watching going; dropping it ends it.
    #[expect(dead_code, reason = "it is held only to be dropped")]
    pub(crate) watching: Box<dyn Any + Send>,
}
