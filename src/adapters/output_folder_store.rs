//! Adapter: the output folders port over the file system.

mod marker_file;

use std::collections::BTreeSet;
use std::fs::{
    create_dir_all, read_dir, read_to_string, remove_dir, remove_dir_all, remove_file,
    symlink_metadata, write,
};
use std::io::{ErrorKind, Result as IoResult};
use std::path::Path;

use marker_file::{marker_text, site_record, versions_record};

use crate::entities::{FilePath, OutputContents, Record};
use crate::ports::OutputFolders;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct OutputFolderStore;

impl OutputFolders for OutputFolderStore {
    fn contents(&self, output: &Path) -> IoResult<OutputContents> {
        let metadata = match symlink_metadata(output) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Ok(OutputContents::Missing);
            }
            Err(error) => return Err(error),
        };
        // A symbolic link is not followed, even to a folder: it is something else's.
        if !metadata.is_dir() {
            return Ok(OutputContents::NotAFolder);
        }
        let mut found = Found::default();
        found.walk(output, "")?;
        let record = match marker(output, Record::VERSIONS_MARKER)? {
            Some(text) => versions_record(&text),
            None => marker(output, Record::SITE_MARKER)?
                .as_deref()
                .and_then(site_record),
        };
        Ok(OutputContents::Folder {
            files: found.files,
            folders: found.folders,
            record,
        })
    }

    fn clear(&self, folder: &Path) -> IoResult<()> {
        create_dir_all(folder)?;
        read_dir(folder)?.try_for_each(|entry| self.remove(&entry?.path()))
    }

    fn remove(&self, path: &Path) -> IoResult<()> {
        let kind = match symlink_metadata(path) {
            Ok(metadata) => metadata.file_type(),
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
        };
        if kind.is_symlink() {
            // The link goes, never what it points to. Windows removes a link
            // to a folder as a folder.
            remove_file(path).or_else(|_| remove_dir(path))
        } else if kind.is_dir() {
            // `remove_dir_all` does not follow the links inside.
            remove_dir_all(path)
        } else {
            remove_file(path)
        }
    }

    fn record(&self, output: &Path, record: &Record) -> IoResult<()> {
        let (name, text) = marker_text(record)?;
        create_dir_all(output)?;
        write(output.join(name), text)
    }
}

/// Every file and folder met while walking a folder.
#[derive(Default)]
struct Found {
    files: BTreeSet<FilePath>,
    folders: BTreeSet<FilePath>,
}

impl Found {
    /// Adds what is in `folder`, whose path below the output folder is
    /// `below`, empty for the output folder itself, never following a link.
    fn walk(&mut self, folder: &Path, below: &str) -> IoResult<()> {
        for entry in read_dir(folder)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let path = if below.is_empty() {
                name.into_owned()
            } else {
                format!("{below}/{name}")
            };
            if entry.file_type()?.is_dir() {
                self.walk(&entry.path(), &path)?;
                self.folders.insert(path);
            } else {
                self.files.insert(path);
            }
        }
        Ok(())
    }
}

/// The text of the marker `name` in `output`, when it is a file there.
fn marker(output: &Path, name: &str) -> IoResult<Option<String>> {
    let path = output.join(name);
    match symlink_metadata(&path) {
        Ok(metadata) if metadata.is_file() => read_to_string(path).map(Some),
        Ok(_) => Ok(None),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}
