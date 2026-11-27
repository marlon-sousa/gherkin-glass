//! Adapter: the output folders port over the file system.

mod marker_file;

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{
    create_dir_all, read_dir, read_to_string, remove_dir, remove_dir_all, remove_file,
    symlink_metadata, write,
};
use std::io::{ErrorKind, Result as IoResult};
use std::path::Path;

use marker_file::{marker_text, site_record, versions_record};

use crate::entities::{EntryKind, FilePath, KeptInOutput, LetterCase, OutputContents, Record};
use crate::ports::OutputFolders;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct OutputFolderStore;

impl OutputFolders for OutputFolderStore {
    fn contents(
        &self,
        output: &Path,
        left_alone: Option<&str>,
        kept: &KeptInOutput,
    ) -> IoResult<OutputContents> {
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
        let mut found = Found {
            left_alone,
            kept_in_output: kept,
            files: BTreeSet::new(),
            folders: BTreeSet::new(),
            kept: BTreeMap::new(),
        };
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
            kept: found.kept,
            record,
        })
    }

    fn read(&self, output: &Path, file: &str) -> IoResult<Option<String>> {
        let path = output.join(file);
        if symlink_metadata(&path)?.is_file() {
            read_to_string(path).map(Some)
        } else {
            Ok(None)
        }
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

/// Every file and folder met while walking a folder, but the folder at its
/// top named `left_alone`, and what `kept_in_output` keeps, which is in `kept`.
struct Found<'a> {
    left_alone: Option<&'a str>,
    kept_in_output: &'a KeptInOutput,
    files: BTreeSet<FilePath>,
    folders: BTreeSet<FilePath>,
    kept: BTreeMap<FilePath, EntryKind>,
}

impl Found<'_> {
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
            let is_folder = entry.file_type()?.is_dir();
            if is_folder && self.left_alone == Some(path.as_str()) {
                continue;
            }
            let kind = if is_folder {
                EntryKind::Folder
            } else {
                EntryKind::File
            };
            if self
                .kept_in_output
                .keeping(&path, kind, LetterCase::AsWritten)
                .is_some()
            {
                self.kept.insert(path, kind);
                continue;
            }
            if is_folder {
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

#[cfg(test)]
mod tests {
    use std::fs::create_dir_all;

    use super::*;
    use crate::test_support::scratch;

    #[test]
    fn only_a_file_is_read_never_a_folder_nor_a_link() {
        let output = scratch("output-folder-store-read");
        create_dir_all(output.join("data/versions/v1.json")).expect("a folder");
        write(output.join("data/site.json"), "{}").expect("written");
        let store = OutputFolderStore;
        assert_eq!(
            store.read(&output, "data/site.json").ok(),
            Some(Some("{}".to_string()))
        );
        assert_eq!(
            store.read(&output, "data/versions/v1.json").ok(),
            Some(None)
        );
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(output.join("data/site.json"), output.join("link.json"))
                .expect("a link");
            assert_eq!(store.read(&output, "link.json").ok(), Some(None));
        }
    }
}
