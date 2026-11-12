//! Port: writing the files of a site, one at a time, wherever a build's destination is.

use std::io::Result as IoResult;
use std::path::Path;

use crate::entities::ServedSite;

pub(crate) trait FileWriter: Send + Sync {
    /// Writes `contents` at `file`, a `/`-joined path below `output`, making
    /// the folders it needs and replacing a file already there.
    fn write(&self, output: &Path, file: &str, contents: &[u8]) -> IoResult<()>;

    /// The site written below `output` since the last call, as it is served,
    /// which starts the next site empty.
    fn site(&self, output: &Path) -> ServedSite;
}
