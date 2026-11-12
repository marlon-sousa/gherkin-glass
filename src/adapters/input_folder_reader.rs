//! Adapter: the input folders port, listing folders and reading files on disk.

use std::fs::{DirEntry, FileType, read_dir, read_to_string};
use std::io::Result as IoResult;
use std::path::Path;

use crate::entities::{EntryKind, FolderEntry};
use crate::ports::InputFolders;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct InputFolderReader;

impl InputFolders for InputFolderReader {
    fn list(&self, folder: &Path) -> IoResult<Vec<FolderEntry>> {
        read_dir(folder)?.map(folder_entry).collect()
    }

    fn read(&self, file: &Path) -> IoResult<String> {
        read_to_string(file)
    }
}

fn folder_entry(entry: IoResult<DirEntry>) -> IoResult<FolderEntry> {
    let entry = entry?;
    Ok(FolderEntry {
        kind: entry_kind(entry.file_type()?),
        path: entry.path(),
    })
}

/// A `DirEntry`'s type is that of the entry itself, so a symbolic link is not followed.
fn entry_kind(kind: FileType) -> EntryKind {
    if kind.is_file() {
        EntryKind::File
    } else if kind.is_dir() {
        EntryKind::Folder
    } else {
        EntryKind::Other
    }
}
