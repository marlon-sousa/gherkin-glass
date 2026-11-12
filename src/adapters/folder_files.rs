//! Adapter: the file writer port over the file system, writing into the output folder.

use std::fs::{create_dir_all, write};
use std::io::Result as IoResult;
use std::path::Path;

use crate::entities::ServedSite;
use crate::ports::FileWriter;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct FolderFiles;

impl FileWriter for FolderFiles {
    fn write(&self, output: &Path, file: &str, contents: &[u8]) -> IoResult<()> {
        let path = file
            .split('/')
            .fold(output.to_path_buf(), |path, part| path.join(part));
        if let Some(folder) = path.parent() {
            create_dir_all(folder)?;
        }
        write(path, contents)
    }

    fn site(&self, output: &Path) -> ServedSite {
        ServedSite::Folder(output.to_path_buf())
    }
}
