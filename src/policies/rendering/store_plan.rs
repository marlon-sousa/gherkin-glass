//! Policy: the store plan, the whole write of a command into its output folder, decided from what the folder's marker lists, what it holds and what its store holds, and what the command writes or removes: the marker first, the files to write in their order, what nothing accounts for any more, to remove after them, and the marker again.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

mod index_data;
mod removals;
mod store_change;

use crate::entities::{
    BuildTarget, FilePath, OutputMarker, PublishedDescriptor, PublishedFeature, PublishedStore,
    SiteAddress, SiteFile, StorePlan, VersionName, VersionsPageRecord,
};
use crate::policies::rendering::{
    PAGES_INDEX, ROOT_PAGE, VERSIONS_LIST, asset_paths, descriptor_path, object_path, shell_path,
};
use index_data::{pages_json, versions_json};
use removals::removals;
pub(crate) use store_change::{
    FolderBefore, StoreChange, VersionsPageWrite, WrittenTarget, versions_once_changed,
};

/// How to make `change` to the folder `before` describes. A build writes the objects the store
/// does not hold yet, this ghg's assets when the folder lacks them, the shell and the
/// descriptor, then, with the versions page, the list of versions and the index of pages, from
/// every descriptor once the change is made. Whatever the marker lists that no descriptor, the
/// list of versions and the files made from them account for once the change is made is
/// removed after them, each descriptor first, then the folders of the versions and previews
/// that no descriptor names and of the assets that nothing links.
pub(crate) fn plan_store(before: &FolderBefore<'_>, change: StoreChange) -> StorePlan {
    let published = before.published;
    let mut descriptors = published.descriptors.clone();
    let mut new_objects = Vec::new();
    let mut target_files = Vec::new();
    match change.written {
        Some(written) => {
            let held = held_objects(published);
            new_objects = written
                .store
                .objects
                .into_iter()
                .filter(|object| !is_in(object, &held))
                .collect();
            descriptors.insert(
                change.target.clone(),
                PublishedDescriptor {
                    ghg: written.ghg,
                    features: written.store.features,
                },
            );
            target_files = vec![written.shell, written.store.descriptor];
        }
        None => {
            descriptors.remove(&change.target);
        }
    }
    let versions = versions_of(&descriptors);
    let (page_record, derived) = match change.versions_page {
        _ if versions.is_empty() => (None, Vec::new()),
        Some(write) => {
            let derived = vec![
                json_file(VERSIONS_LIST, versions_json(&versions, &write.record)),
                json_file(PAGES_INDEX, pages_json(&versions)),
                write.page,
            ];
            (Some(write.record), derived)
        }
        None => (published.versions_page.clone(), Vec::new()),
    };
    let accounted = accounted(&descriptors, page_record.as_ref());
    let assets: Vec<SiteFile> = change
        .assets
        .into_iter()
        .filter(|asset| accounted.contains(asset.address.as_str()))
        .collect();
    let assets = if assets.iter().all(|asset| is_in(asset, before.found)) {
        Vec::new()
    } else {
        assets
    };
    let leftover: BTreeSet<FilePath> = before
        .listed
        .difference(&accounted)
        .filter(|file| file.as_str() != OutputMarker::FILE)
        .cloned()
        .collect();
    StorePlan {
        first_marker: accounted.union(&leftover).cloned().collect(),
        writes: new_objects
            .into_iter()
            .chain(assets)
            .chain(target_files)
            .chain(derived)
            .collect(),
        removes: removals(&leftover, &descriptors, page_record.as_ref()),
        last_marker: accounted,
    }
}

/// Whether `file` is among `files`.
fn is_in(file: &SiteFile, files: &BTreeSet<FilePath>) -> bool {
    files.contains(file.address.as_str())
}

/// The paths of the objects the store of `published` holds.
fn held_objects(published: &PublishedStore) -> BTreeSet<FilePath> {
    published
        .objects
        .iter()
        .map(|name| object_path(name))
        .collect()
}

/// Each version's features, by its name, among `descriptors`.
fn versions_of(
    descriptors: &BTreeMap<BuildTarget, PublishedDescriptor>,
) -> BTreeMap<VersionName, Vec<PublishedFeature>> {
    descriptors
        .iter()
        .filter_map(|(target, descriptor)| {
            Some((target.version()?.clone(), descriptor.features.clone()))
        })
        .collect()
}

/// Every file the store and the files made from it account for: each descriptor, the objects
/// it names, its shell and its ghg's assets, and, beside versions, the list of versions, the
/// index of pages, the versions page and the assets it links.
fn accounted(
    descriptors: &BTreeMap<BuildTarget, PublishedDescriptor>,
    page: Option<&VersionsPageRecord>,
) -> BTreeSet<FilePath> {
    descriptors
        .iter()
        .flat_map(|(target, descriptor)| files_of_descriptor(target, descriptor))
        .chain(page.into_iter().flat_map(files_of_versions_page))
        .collect()
}

/// The descriptor of `target`, its shell, the objects it names and its ghg's assets.
fn files_of_descriptor(target: &BuildTarget, descriptor: &PublishedDescriptor) -> Vec<FilePath> {
    let objects = descriptor
        .features
        .iter()
        .map(|feature| object_path(&feature.object));
    [descriptor_path(target), shell_path(target)]
        .into_iter()
        .chain(objects)
        .chain(asset_paths(&descriptor.ghg))
        .collect()
}

/// The list of versions, the index of pages, the versions page and the assets of the ghg that
/// wrote it, as `record` says.
fn files_of_versions_page(record: &VersionsPageRecord) -> Vec<FilePath> {
    [VERSIONS_LIST, PAGES_INDEX, ROOT_PAGE]
        .into_iter()
        .map(str::to_string)
        .chain(asset_paths(&record.ghg))
        .collect()
}

fn json_file(path: &str, bytes: Vec<u8>) -> SiteFile {
    SiteFile {
        address: SiteAddress::at_root(path),
        contents: Arc::from(bytes),
    }
}

#[cfg(test)]
mod tests;
