//! Port: an output folder, what is in it, the files of its store and its marker, and removing what ghg wrote there.

use std::io::Result as IoResult;
use std::path::Path;

use crate::entities::{KeptInOutput, OutputContents};

pub(crate) trait OutputFolders: Send + Sync {
    /// What is at `output`'s place: every file and folder below it, and the text of its marker;
    /// what `kept` keeps is listed apart, and a folder kept is not looked into.
    fn contents(&self, output: &Path, kept: &KeptInOutput) -> IoResult<OutputContents>;

    /// The text of the file at `file`, a `/`-joined path below `output`, which is
    /// read only when it is a file, never through a link; `None` when it is a folder or a
    /// link.
    fn read(&self, output: &Path, file: &str) -> IoResult<Option<String>>;

    /// Removes what is at `path`, a `/`-joined path below `output`, a file, a link or a folder
    /// with all in it, never following a link, then each folder that held it and is left
    /// empty, but `output`; nothing when there is nothing there.
    fn remove(&self, output: &Path, path: &str) -> IoResult<()>;
}
