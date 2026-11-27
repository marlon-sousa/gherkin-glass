//! Policy: the files of the site's store rendered as compact JSON, each object named by the SHA-256 of its bytes, where each file of the store goes, and what the output folder's store already holds.

mod descriptor_data;
mod index_data;
mod object_data;
mod published_data;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use serde::Serialize;
use sha2::{Digest, Sha256};

pub(crate) use descriptor_data::DescriptorSettings;
use descriptor_data::descriptor_json;
pub(super) use index_data::{pages_json, versions_json};
use object_data::object_json;
pub(crate) use published_data::published_features;

use crate::entities::{
    DescriptorFeature, FilePath, PublishedFeature, SiteAddress, SiteFile, SiteStore, StoreFiles,
    VersionName,
};

/// The number of the store's format, which every file of the store holds.
const STORE_FORMAT: u32 = 1;

/// The version of the ghg that writes the store.
const GHG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Where the objects are, below the output folder.
const OBJECTS: &str = "data/objects/";
/// Where the versions' descriptors are, below the output folder.
const VERSIONS: &str = "data/versions/";
const SITE_DESCRIPTOR: &str = "data/site.json";
pub(super) const VERSIONS_LIST: &str = "data/versions.json";
pub(super) const PAGES_INDEX: &str = "data/pages.json";
const JSON: &str = ".json";

/// What the output folder's store holds, by the paths of its files: the descriptor of each
/// version, and the name of each object.
pub(crate) struct StoreListing {
    pub(crate) descriptors: Vec<(VersionName, FilePath)>,
    pub(crate) objects: BTreeSet<String>,
}

/// The store's files of `store`: each distinct object, named by the SHA-256 of its bytes, and
/// the descriptor, of the version `settings` names or of a site without versions.
pub(crate) fn render_store(store: &SiteStore, settings: &DescriptorSettings<'_>) -> StoreFiles {
    let rendered: Vec<(String, Vec<u8>)> = store
        .features
        .iter()
        .map(|feature| named(object_json(&feature.object)))
        .collect();
    let names: Vec<String> = rendered.iter().map(|(name, _)| name.clone()).collect();
    let objects: BTreeMap<String, Vec<u8>> = rendered.into_iter().collect();
    StoreFiles {
        objects: objects.into_iter().map(object_file).collect(),
        descriptor: SiteFile {
            address: SiteAddress::at_root(&descriptor_path(settings.version)),
            contents: Arc::from(descriptor_json(store, &names, settings)),
        },
        features: store
            .features
            .iter()
            .zip(names)
            .map(published_feature)
            .collect(),
    }
}

/// The descriptors and objects among `files`, every file below the output folder.
pub(crate) fn store_listing(files: &BTreeSet<FilePath>) -> StoreListing {
    StoreListing {
        descriptors: files.iter().filter_map(version_descriptor).collect(),
        objects: files.iter().filter_map(object_name).collect(),
    }
}

/// The path below the output folder of the descriptor of `version`, or of a site without
/// versions.
pub(super) fn descriptor_path(version: Option<&VersionName>) -> FilePath {
    match version {
        Some(name) => format!("{VERSIONS}{name}{JSON}"),
        None => SITE_DESCRIPTOR.to_string(),
    }
}

/// The path below the output folder of the object named `name`.
pub(super) fn object_path(name: &str) -> FilePath {
    format!("{OBJECTS}{name}{JSON}")
}

/// `bytes`, named by their SHA-256.
fn named(bytes: Vec<u8>) -> (String, Vec<u8>) {
    (sha256_hex(&bytes), bytes)
}

fn object_file((name, bytes): (String, Vec<u8>)) -> SiteFile {
    SiteFile {
        address: SiteAddress::at_root(&object_path(&name)),
        contents: Arc::from(bytes),
    }
}

fn published_feature((feature, object): (&DescriptorFeature, String)) -> PublishedFeature {
    PublishedFeature {
        page: feature.page.clone(),
        object,
    }
}

/// The version whose descriptor is at `path`, when it is one.
fn version_descriptor(path: &FilePath) -> Option<(VersionName, FilePath)> {
    let name = path.strip_prefix(VERSIONS)?.strip_suffix(JSON)?;
    Some((VersionName::new(name)?, path.clone()))
}

/// The name of the object at `path`, when it is one.
fn object_name(path: &FilePath) -> Option<String> {
    let name = path.strip_prefix(OBJECTS)?.strip_suffix(JSON)?;
    (!name.contains('/')).then(|| name.to_string())
}

/// The SHA-256 of `bytes`, in lower-case hexadecimal.
fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(hex_pair).collect()
}

fn hex_pair(byte: &u8) -> String {
    format!("{byte:02x}")
}

/// `data` as compact JSON, as rule 247 writes every file of the store.
#[expect(
    clippy::expect_used,
    reason = "structs of strings, booleans, whole numbers, lists and options of them always serialise"
)]
fn json_bytes(data: &impl Serialize) -> Vec<u8> {
    serde_json::to_vec(data).expect(
        "structs of strings, booleans, whole numbers, lists and options of them always serialise",
    )
}

#[cfg(test)]
mod tests;
