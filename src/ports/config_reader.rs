//! Port: reading a configuration file's text.

use std::io::Result as IoResult;
use std::path::Path;

pub(crate) trait ConfigReader: Send + Sync {
    /// An error of kind `NotFound` when there is no file at `path`.
    fn read(&self, path: &Path) -> IoResult<String>;
}
