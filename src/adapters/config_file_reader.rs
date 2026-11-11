//! Adapter: the config reader port over the file system.

use std::fs::read_to_string;
use std::io::Result as IoResult;
use std::path::Path;

use crate::ports::ConfigReader;

#[derive(Debug, Default, Clone, Copy)]
pub struct ConfigFileReader;

impl ConfigReader for ConfigFileReader {
    fn read(&self, path: &Path) -> IoResult<String> {
        read_to_string(path)
    }
}
