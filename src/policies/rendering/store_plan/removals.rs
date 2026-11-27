//! Policy: what the store plan removes of what nothing accounts for any more, and in what order.

use std::collections::{BTreeMap, BTreeSet};

use crate::entities::{
    BuildTarget, FilePath, PublishedDescriptor, VersionName, VersionsPageRecord,
};
use crate::policies::rendering::{
    ASSETS, DATA, PREVIEWS, SITE_DESCRIPTOR, VERSIONS, assets_folder,
};

/// What removing `leftover` removes, in order: each descriptor first, then every other file,
/// each within the folder of a version, a preview or a ghg's assets that nothing accounts for
/// any more removed with that folder.
pub(super) fn removals(
    leftover: &BTreeSet<FilePath>,
    descriptors: &BTreeMap<BuildTarget, PublishedDescriptor>,
    page: Option<&VersionsPageRecord>,
) -> Vec<FilePath> {
    let still_there = folders_still_there(descriptors, page);
    let (descriptors, others): (Vec<&FilePath>, Vec<&FilePath>) =
        leftover.iter().partition(|file| is_descriptor(file));
    let others: BTreeSet<FilePath> = others
        .into_iter()
        .map(|file| removed_with(file, &still_there))
        .collect();
    descriptors.into_iter().cloned().chain(others).collect()
}

/// The folders of the versions and previews `descriptors` name, and of the assets they and the
/// versions page of `page` link.
fn folders_still_there(
    descriptors: &BTreeMap<BuildTarget, PublishedDescriptor>,
    page: Option<&VersionsPageRecord>,
) -> BTreeSet<FilePath> {
    let of_descriptors = descriptors.iter().flat_map(|(target, descriptor)| {
        target
            .folder()
            .into_iter()
            .chain([assets_folder(&descriptor.ghg)])
    });
    of_descriptors
        .chain(page.map(|record| assets_folder(&record.ghg)))
        .collect()
}

/// What removes `file`: the folder it is in, when nothing of `still_there` is that folder, or
/// else `file` alone.
fn removed_with(file: &FilePath, still_there: &BTreeSet<FilePath>) -> FilePath {
    match removed_folder(file) {
        Some(folder) if !still_there.contains(&folder) => folder,
        Some(_) | None => file.clone(),
    }
}

fn is_descriptor(file: &str) -> bool {
    file == SITE_DESCRIPTOR || file.starts_with(VERSIONS) || file.starts_with(PREVIEWS)
}

/// The folder of a version, a preview or a ghg's assets that `file` is in, if any:
/// `<version>/`, `previews/<name>/` or `assets/<ghg>/`.
fn removed_folder(file: &str) -> Option<FilePath> {
    let (top, rest) = file.split_once('/')?;
    match top {
        DATA => None,
        ASSETS | VersionName::PREVIEWS => rest
            .split_once('/')
            .map(|(inner, _)| format!("{top}/{inner}")),
        _ => Some(top.to_string()),
    }
}
