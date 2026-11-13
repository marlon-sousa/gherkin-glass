//! Port: reading a file's text, a configuration file or a front page's template.

use std::io::Result as IoResult;
use std::path::Path;

pub(crate) trait FileReader: Send + Sync {
    /// An error of kind `NotFound` when there is no file at `path`.
    fn read(&self, path: &Path) -> IoResult<String>;
}
