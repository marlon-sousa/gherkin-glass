//! Service: the site writer's reading of what the output folder's store holds, its versions' descriptors through the output folders port and its objects' names from the folder's files.

use std::collections::BTreeSet;
use std::path::Path;

use crate::entities::{FilePath, PublishedFeature, PublishedStore};
use crate::errors::OutputErrorKind;
use crate::policies::{published_features, store_listing};
use crate::ports::OutputFolders;

/// What the store among `files`, every file of `output`, holds; `files` is empty when the
/// build replaces every file it wrote there.
pub(super) fn published_store(
    folders: &dyn OutputFolders,
    output: &Path,
    files: &BTreeSet<FilePath>,
) -> Result<PublishedStore, OutputErrorKind> {
    let listing = store_listing(files);
    let versions = listing
        .descriptors
        .into_iter()
        .map(|(name, path)| Ok((name, read_descriptor(folders, output, &path)?)))
        .collect::<Result<_, OutputErrorKind>>()?;
    Ok(PublishedStore {
        versions,
        objects: listing.objects,
    })
}

/// The features the descriptor at `path` names.
fn read_descriptor(
    folders: &dyn OutputFolders,
    output: &Path,
    path: &str,
) -> Result<Vec<PublishedFeature>, OutputErrorKind> {
    let text = folders
        .read(output, path)
        .map_err(|io| OutputErrorKind::CannotBeRead {
            reason: io.to_string(),
        })?;
    // A descriptor that cannot be read could name objects that would otherwise be removed.
    published_features(&text).ok_or(OutputErrorKind::NotASite)
}
