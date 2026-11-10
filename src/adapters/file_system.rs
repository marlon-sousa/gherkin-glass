//! Adapter: the input folder port over the file system.

use std::fs::{read_dir, read_to_string};
use std::io::Result as IoResult;
use std::path::Path;

use crate::entities::{EntryKind, FolderEntry};
use crate::ports::InputFolder;

#[derive(Debug, Default, Clone, Copy)]
pub struct FileSystem;

impl InputFolder for FileSystem {
    fn entries(&self, folder: &Path) -> IoResult<Vec<FolderEntry>> {
        read_dir(folder)?
            .map(|entry| {
                let entry = entry?;
                // The entry's own type, so a symbolic link is not followed.
                let kind = entry.file_type()?;
                let kind = if kind.is_file() {
                    EntryKind::File
                } else if kind.is_dir() {
                    EntryKind::Folder
                } else {
                    EntryKind::Other
                };
                Ok(FolderEntry {
                    path: entry.path(),
                    kind,
                })
            })
            .collect()
    }

    fn read_text(&self, path: &Path) -> IoResult<String> {
        read_to_string(path)
    }
}
