//! Adapter: the output folders port over the file system.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{
    read_dir, read_to_string, remove_dir, remove_dir_all, remove_file, symlink_metadata,
};
use std::io::{ErrorKind, Result as IoResult};
use std::path::Path;

use crate::entities::{
    EntryKind, FilePath, KeptInOutput, LetterCase, OutputContents, OutputMarker,
};
use crate::ports::OutputFolders;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct OutputFolderStore;

impl OutputFolders for OutputFolderStore {
    fn contents(&self, output: &Path, kept: &KeptInOutput) -> IoResult<OutputContents> {
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
            kept_in_output: kept,
            files: BTreeSet::new(),
            folders: BTreeSet::new(),
            kept: BTreeMap::new(),
        };
        found.walk(output, "")?;
        Ok(OutputContents::Folder {
            files: found.files,
            folders: found.folders,
            kept: found.kept,
            marker: marker(output)?,
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

    fn remove(&self, output: &Path, path: &str) -> IoResult<()> {
        remove_entry(&output.join(path))?;
        let parents = path
            .rmatch_indices('/')
            .filter_map(|(end, _)| path.get(..end));
        for parent in parents {
            // A folder that still holds something stays, with the folders that hold it.
            match remove_dir(output.join(parent)) {
                Ok(()) => {}
                Err(error) if error.kind() == ErrorKind::NotFound => {}
                Err(_) => return Ok(()),
            }
        }
        Ok(())
    }
}

/// Removes what is at `path`, never following a link; nothing when there is nothing there.
fn remove_entry(path: &Path) -> IoResult<()> {
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

/// Every file and folder met while walking a folder, but what `kept_in_output` keeps, which is
/// in `kept`.
struct Found<'a> {
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

/// The text of the marker in `output`, when it is a file there.
fn marker(output: &Path) -> IoResult<Option<String>> {
    let path = output.join(OutputMarker::FILE);
    match symlink_metadata(&path) {
        Ok(metadata) if metadata.is_file() => read_to_string(path).map(Some),
        Ok(_) => Ok(None),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{create_dir_all, write};

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

    #[test]
    fn removing_a_file_removes_the_folders_it_leaves_empty() {
        let output = scratch("output-folder-store-remove");
        create_dir_all(output.join("previews/pr-1")).expect("a folder");
        create_dir_all(output.join("data/previews")).expect("a folder");
        write(output.join("previews/pr-1/index.html"), "").expect("written");
        write(output.join("data/previews/pr-1.json"), "").expect("written");
        write(output.join("data/site.json"), "").expect("written");
        let store = OutputFolderStore;
        store
            .remove(&output, "previews/pr-1/index.html")
            .expect("removed");
        store
            .remove(&output, "data/previews/pr-1.json")
            .expect("removed");
        assert!(!output.join("previews").exists());
        assert!(!output.join("data/previews").exists());
        assert!(output.join("data/site.json").exists());
        assert!(output.exists());
    }
}
