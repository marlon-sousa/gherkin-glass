//! Policy: the store plan, the whole write of the store decided from what the output folder's store holds and the files of the store a build renders: the files to write, what nothing references any more, to remove after them, and every file of the store once it is written.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

mod index_data;

use crate::entities::{
    FilePath, PageOptions, PublishedFeature, PublishedStore, SiteAddress, SiteFile, StoreFiles,
    StorePlan, VersionName,
};
use crate::policies::rendering::{PAGES_INDEX, VERSIONS_LIST, descriptor_path, object_path};
use index_data::{pages_json, versions_json};

/// How to write `built`, the store's files of the version `version`, beside `published`, what
/// the output folder's store holds, or of a site without versions, whose output folder holds
/// nothing of an earlier build by then. A version's build writes the objects the store does
/// not hold yet, its descriptor, and the list of versions and the index of pages, written with
/// the settings of every page `options` and the front page's template, if any, from every
/// version's descriptor, its own in place of the one it replaces; then removes every object
/// that no descriptor names.
pub(crate) fn plan_store(
    published: &PublishedStore,
    built: StoreFiles,
    version: Option<&VersionName>,
    options: &PageOptions,
    front_page: Option<&str>,
) -> StorePlan {
    let Some(version) = version else {
        let writes: Vec<SiteFile> = built
            .objects
            .into_iter()
            .chain([built.descriptor])
            .collect();
        return StorePlan {
            files: writes.iter().map(SiteFile::path).collect(),
            writes,
            removes: BTreeSet::new(),
        };
    };
    let mut versions = published.versions.clone();
    versions.insert(version.clone(), built.features);
    let referenced: BTreeSet<&str> = versions
        .values()
        .flatten()
        .map(|feature| feature.object.as_str())
        .collect();
    let held: BTreeSet<FilePath> = published
        .objects
        .iter()
        .map(|name| object_path(name))
        .collect();
    let new_objects = built
        .objects
        .into_iter()
        .filter(|object| is_new(object, &held));
    let indexes = [
        json_file(VERSIONS_LIST, versions_json(&versions, options, front_page)),
        json_file(PAGES_INDEX, pages_json(&versions)),
    ];
    let writes = new_objects
        .chain([built.descriptor])
        .chain(indexes)
        .collect();
    StorePlan {
        writes,
        removes: unreferenced(&published.objects, &referenced),
        files: store_files(&versions, &referenced),
    }
}

/// The paths of the objects of `objects` that `referenced` does not name.
fn unreferenced(objects: &BTreeSet<String>, referenced: &BTreeSet<&str>) -> BTreeSet<FilePath> {
    objects
        .iter()
        .filter(|name| is_unreferenced(name, referenced))
        .map(|name| object_path(name))
        .collect()
}

/// Every file of a versioned site's store: each version's descriptor, the objects they name,
/// the list of versions and the index of pages.
fn store_files(
    versions: &BTreeMap<VersionName, Vec<PublishedFeature>>,
    referenced: &BTreeSet<&str>,
) -> BTreeSet<FilePath> {
    versions
        .keys()
        .map(Some)
        .map(descriptor_path)
        .chain(referenced.iter().map(|name| object_path(name)))
        .chain([VERSIONS_LIST.to_string(), PAGES_INDEX.to_string()])
        .collect()
}

/// Whether the store does not hold `object` yet.
fn is_new(object: &SiteFile, held: &BTreeSet<FilePath>) -> bool {
    !held.contains(object.address.as_str())
}

fn is_unreferenced(name: &str, referenced: &BTreeSet<&str>) -> bool {
    !referenced.contains(name)
}

fn json_file(path: &str, bytes: Vec<u8>) -> SiteFile {
    SiteFile {
        address: SiteAddress::at_root(path),
        contents: Arc::from(bytes),
    }
}

#[cfg(test)]
mod tests;
