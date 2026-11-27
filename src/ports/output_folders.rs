//! Port: an output folder, what is in it, the files of its store, and its marker.

use std::io::Result as IoResult;
use std::path::Path;

use crate::entities::{KeptInOutput, OutputContents, Record};

pub(crate) trait OutputFolders: Send + Sync {
    /// What is at `output`'s place: every file and folder below it, and what
    /// its marker records, but the folder `left_alone` at its top, when there is
    /// one, which is neither looked into nor listed; what `kept` keeps is
    /// listed apart, and a folder kept is not looked into.
    fn contents(
        &self,
        output: &Path,
        left_alone: Option<&str>,
        kept: &KeptInOutput,
    ) -> IoResult<OutputContents>;

    /// The text of the file at `file`, a `/`-joined path below `output`, which is
    /// read only when it is a file, never through a link; `None` when it is a folder or a
    /// link.
    fn read(&self, output: &Path, file: &str) -> IoResult<Option<String>>;

    /// Removes what is at `path`, a file, a link or a folder with all in it,
    /// never following a link; nothing when there is nothing there.
    fn remove(&self, path: &Path) -> IoResult<()>;

    /// Writes `record` as the marker of `output`, creating the folder when missing.
    fn record(&self, output: &Path, record: &Record) -> IoResult<()>;
}
