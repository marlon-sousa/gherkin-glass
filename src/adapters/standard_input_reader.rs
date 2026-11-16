//! Adapter: the standard input port over the process's standard input, read once.

use std::io::{Error as IoError, ErrorKind, Read, Result as IoResult, stdin};
use std::sync::OnceLock;

use crate::ports::StandardInput;

#[derive(Debug, Default)]
pub(crate) struct StandardInputReader {
    /// What the one read gave: the text, or the kind and the words of its error.
    read: OnceLock<Result<String, (ErrorKind, String)>>,
}

impl StandardInput for StandardInputReader {
    fn read(&self) -> IoResult<String> {
        self.read.get_or_init(read_once).clone().map_err(io_error)
    }
}

fn read_once() -> Result<String, (ErrorKind, String)> {
    let mut text = String::new();
    stdin()
        .lock()
        .read_to_string(&mut text)
        .map(|_| text)
        .map_err(kept_error)
}

fn kept_error(error: IoError) -> (ErrorKind, String) {
    (error.kind(), error.to_string())
}

fn io_error((kind, words): (ErrorKind, String)) -> IoError {
    IoError::new(kind, words)
}
