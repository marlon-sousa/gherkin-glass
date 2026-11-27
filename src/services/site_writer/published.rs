//! Service: the site writer's reading of what the output folder's store holds, its descriptors and its list of versions through the output folders port, and its objects' names from the folder's files.

use std::collections::BTreeSet;
use std::path::Path;

use crate::entities::{FilePath, PublishedStore};
use crate::errors::OutputErrorKind;
use crate::policies::StoreListing;
use crate::ports::OutputFolders;

/// What the store among `files`, every file of `output` its marker lists, holds. A descriptor
/// or a list of versions that is a folder or a link, or not of the store's format, makes the
/// folder no site ghg built, since it could name objects that would otherwise be removed.
pub(super) fn published_store(
    folders: &dyn OutputFolders,
    output: &Path,
    files: &BTreeSet<FilePath>,
) -> Result<PublishedStore, OutputErrorKind> {
    let listing = StoreListing::of(files);
    let texts = listing
        .to_read()
        .map(|path| read_descriptor(folders, output, path))
        .collect::<Result<Option<Vec<_>>, _>>()?
        .ok_or(OutputErrorKind::NotASite)?;
    listing.published(&texts).ok_or(OutputErrorKind::NotASite)
}

fn read_descriptor(
    folders: &dyn OutputFolders,
    output: &Path,
    path: &str,
) -> Result<Option<String>, OutputErrorKind> {
    folders
        .read(output, path)
        .map_err(|io| OutputErrorKind::CannotBeRead {
            reason: io.to_string(),
        })
}
