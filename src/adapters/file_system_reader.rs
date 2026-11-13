//! Adapter: the file reader port over the file system.

use std::fs::read_to_string;
use std::io::Result as IoResult;
use std::path::Path;

use crate::ports::FileReader;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct FileSystemReader;

impl FileReader for FileSystemReader {
    fn read(&self, path: &Path) -> IoResult<String> {
        read_to_string(path)
    }
}
