//! Port: writing the files of a site.

use std::io::Result as IoResult;
use std::path::Path;

pub(crate) trait SiteWriter: Send + Sync {
    /// Writes `contents` at `file`, a `/`-joined path below `output`, making
    /// the folders it needs and replacing a file already there.
    fn write(&self, output: &Path, file: &str, contents: &[u8]) -> IoResult<()>;
}
