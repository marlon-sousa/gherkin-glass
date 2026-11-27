//! Policy: the store plan, the whole write of a command into its output folder, decided from what the folder's marker lists, what it holds and what its store holds, and what the command writes or removes: the marker first, the files to write in their order, what nothing accounts for any more, to remove after them, and the marker again.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

mod index_data;

use crate::entities::{
    BuildTarget, FilePath, OutputMarker, PublishedDescriptor, PublishedFeature, PublishedStore,
    SiteAddress, SiteFile, StoreFiles, StorePlan, VersionName, VersionsPageRecord,
};
use crate::policies::rendering::{
    ASSETS, DATA, PAGES_INDEX, PREVIEWS, ROOT_PAGE, SITE_DESCRIPTOR, VERSIONS, VERSIONS_LIST,
    asset_paths, assets_folder, descriptor_path, object_path, shell_path,
};
use index_data::{pages_json, versions_json};

/// What a command does to the output folder's store.
pub(crate) struct StoreChange {
    pub(crate) target: BuildTarget,
    /// `None` when the target is removed.
    pub(crate) written: Option<WrittenTarget>,
    /// The versions page, written again when a version is added or removed and another stays.
    pub(crate) versions_page: Option<VersionsPageWrite>,
    /// The files of this ghg's assets.
    pub(crate) assets: Vec<SiteFile>,
}

/// The files of a target a build writes.
pub(crate) struct WrittenTarget {
    pub(crate) store: StoreFiles,
    pub(crate) shell: SiteFile,
    /// The version of the ghg that writes it.
    pub(crate) ghg: String,
}

/// The versions page, rendered, and what it is written with.
pub(crate) struct VersionsPageWrite {
    pub(crate) record: VersionsPageRecord,
    pub(crate) page: SiteFile,
}

/// What the folder holds before the command writes anything.
pub(crate) struct FolderBefore<'a> {
    /// Every file its marker lists.
    pub(crate) listed: &'a BTreeSet<FilePath>,
    /// Every file in it, but what is kept.
    pub(crate) found: &'a BTreeSet<FilePath>,
    pub(crate) published: &'a PublishedStore,
}

/// The versions the store holds once `target` is written, or removed when `removed`.
pub(crate) fn versions_once_changed(
    published: &PublishedStore,
    target: &BuildTarget,
    removed: bool,
) -> Vec<VersionName> {
    let mut targets: BTreeSet<&BuildTarget> = published.descriptors.keys().collect();
    if removed {
        targets.remove(target);
    } else {
        targets.insert(target);
    }
    targets
        .into_iter()
        .filter_map(BuildTarget::version)
        .cloned()
        .collect()
}

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
            new_objects = written
                .store
                .objects
                .into_iter()
                .filter(|object| is_new(object, &published.objects))
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
    let lacks_assets = assets
        .iter()
        .any(|asset| !before.found.contains(asset.address.as_str()));
    let assets = assets.into_iter().filter(|_| lacks_assets);
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

/// Whether the store does not hold `object` yet, among `held`, the names of its objects.
fn is_new(object: &SiteFile, held: &BTreeSet<String>) -> bool {
    !held
        .iter()
        .any(|name| object_path(name) == object.address.as_str())
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
    let of_descriptors = descriptors.iter().flat_map(|(target, descriptor)| {
        [descriptor_path(target), shell_path(target)]
            .into_iter()
            .chain(
                descriptor
                    .features
                    .iter()
                    .map(|feature| object_path(&feature.object)),
            )
            .chain(asset_paths(&descriptor.ghg).collect::<Vec<_>>())
    });
    let of_page = page.into_iter().flat_map(|record| {
        [
            VERSIONS_LIST.to_string(),
            PAGES_INDEX.to_string(),
            ROOT_PAGE.to_string(),
        ]
        .into_iter()
        .chain(asset_paths(&record.ghg).collect::<Vec<_>>())
    });
    of_descriptors.chain(of_page).collect()
}

/// What removing `leftover` removes, in order: each descriptor first, then every other file,
/// each within the folder of a version, a preview or a ghg's assets that nothing accounts for
/// any more removed with that folder.
fn removals(
    leftover: &BTreeSet<FilePath>,
    descriptors: &BTreeMap<BuildTarget, PublishedDescriptor>,
    page: Option<&VersionsPageRecord>,
) -> Vec<FilePath> {
    let still_there: BTreeSet<FilePath> = descriptors
        .iter()
        .flat_map(|(target, descriptor)| {
            target
                .folder()
                .into_iter()
                .chain([assets_folder(&descriptor.ghg)])
        })
        .chain(page.map(|record| assets_folder(&record.ghg)))
        .collect();
    let (descriptors, others): (Vec<&FilePath>, Vec<&FilePath>) =
        leftover.iter().partition(|file| is_descriptor(file));
    let others: BTreeSet<FilePath> = others
        .into_iter()
        .map(|file| {
            removed_folder(file)
                .filter(|folder| !still_there.contains(folder))
                .unwrap_or_else(|| file.clone())
        })
        .collect();
    descriptors.into_iter().cloned().chain(others).collect()
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

fn json_file(path: &str, bytes: Vec<u8>) -> SiteFile {
    SiteFile {
        address: SiteAddress::at_root(path),
        contents: Arc::from(bytes),
    }
}

#[cfg(test)]
mod tests;
