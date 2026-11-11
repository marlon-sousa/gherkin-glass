//! Adapter: the output folders port over the file system.

use std::fs::{
    create_dir_all, read_dir, remove_dir, remove_dir_all, remove_file, symlink_metadata, write,
};
use std::io::{ErrorKind, Result as IoResult};
use std::path::Path;

use crate::entities::{FolderState, Marker, VersionName};
use crate::ports::OutputFolders;

/// The marker in every version's folder, docs/BEHAVIOUR.md, rule 122.
pub(crate) const VERSION_MARKER: &str = "ghg-version.json";

/// The marker in an output folder holding a site without versions, rule 38.
pub(crate) const SITE_MARKER: &str = "ghg-site.json";

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct OutputFolderStore;

impl OutputFolders for OutputFolderStore {
    fn state(&self, folder: &Path, marker: &Marker) -> IoResult<FolderState> {
        let metadata = match symlink_metadata(folder) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(FolderState::Missing),
            Err(error) => return Err(error),
        };
        // A symbolic link is not followed, even to a folder: it is something else's.
        if !metadata.is_dir() {
            return Ok(FolderState::Other);
        }
        let file = match marker {
            Marker::Site => SITE_MARKER,
            // On a file system that does not tell capitals apart, `Main` finds
            // `main`. An output folder is the one named, in any capitals.
            Marker::Version(_) if !named_exactly(folder)? => {
                return Ok(FolderState::OtherCapitals);
            }
            Marker::Version(_) => VERSION_MARKER,
        };
        if is_file(&folder.join(file))? {
            return Ok(FolderState::Marked);
        }
        match read_dir(folder)?.next() {
            None => Ok(FolderState::Empty),
            Some(entry) => entry.map(|_| FolderState::Other),
        }
    }

    fn clear(&self, folder: &Path) -> IoResult<()> {
        create_dir_all(folder)?;
        for entry in read_dir(folder)? {
            self.remove(&entry?.path())?;
        }
        Ok(())
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

    fn mark(&self, folder: &Path, marker: &Marker) -> IoResult<()> {
        // A version name holds no character JSON must escape.
        match marker {
            Marker::Site => write(folder.join(SITE_MARKER), "{\"site\": true}\n"),
            Marker::Version(name) => write(
                folder.join(VERSION_MARKER),
                format!("{{\"version\": \"{name}\"}}\n"),
            ),
        }
    }

    fn marked(&self, output: &Path) -> IoResult<Vec<VersionName>> {
        // A file or a link in the output folder's place holds no versions.
        match symlink_metadata(output) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => return Ok(Vec::new()),
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error),
        }
        let entries = read_dir(output)?;
        let mut names = Vec::new();
        for entry in entries {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            // A folder whose name cannot be a version name was not built by ghg.
            let Some(name) = entry.file_name().to_str().and_then(VersionName::new) else {
                continue;
            };
            if is_file(&entry.path().join(VERSION_MARKER))? {
                names.push(name);
            }
        }
        Ok(names)
    }
}

/// Whether the folder that holds `path` lists it under exactly its name.
fn named_exactly(path: &Path) -> IoResult<bool> {
    let (Some(parent), Some(name)) = (path.parent(), path.file_name()) else {
        return Ok(true);
    };
    // `Path::parent` of a bare name is the empty path, the current folder.
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    for entry in read_dir(parent)? {
        if entry?.file_name() == name {
            return Ok(true);
        }
    }
    Ok(false)
}

fn is_file(path: &Path) -> IoResult<bool> {
    match symlink_metadata(path) {
        Ok(metadata) => Ok(metadata.is_file()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}
