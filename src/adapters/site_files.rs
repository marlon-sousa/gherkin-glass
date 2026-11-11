//! Adapter: the site writer port over the file system.

use std::fs::{create_dir_all, write};
use std::io::Result as IoResult;
use std::path::Path;

use crate::ports::SiteWriter;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct SiteFiles;

impl SiteWriter for SiteFiles {
    fn write(&self, output: &Path, file: &str, contents: &[u8]) -> IoResult<()> {
        let path = file
            .split('/')
            .fold(output.to_path_buf(), |path, part| path.join(part));
        if let Some(folder) = path.parent() {
            create_dir_all(folder)?;
        }
        write(path, contents)
    }
}
