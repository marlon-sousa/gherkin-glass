//! Port: reading the text of standard input, the list `--only -` reads.

use std::io::Result as IoResult;

pub(crate) trait StandardInput: Send + Sync {
    /// The whole of standard input, the same text every time, since it can be read only once.
    fn read(&self) -> IoResult<String>;
}
