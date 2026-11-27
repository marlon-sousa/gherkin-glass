//! Policy: what the output folder's store already holds: the descriptor of each version, by the paths of the folder's files, the features each descriptor names, read from its JSON, and the objects there.

use std::collections::BTreeSet;

use serde::Deserialize;

use crate::entities::{FilePath, PublishedFeature, PublishedStore, VersionName};
use crate::policies::rendering::{JSON, OBJECTS, STORE_FORMAT, VERSIONS};

/// The store among the files of an output folder: the descriptor of each version, and the
/// name of each object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StoreListing {
    descriptors: Vec<(VersionName, FilePath)>,
    objects: BTreeSet<String>,
}

/// A descriptor of the store, but for the members the site writer does not read.
#[derive(Deserialize)]
struct PublishedData {
    format: u32,
    features: Vec<FeatureData>,
}

#[derive(Deserialize)]
struct FeatureData {
    page: String,
    object: String,
}

impl StoreListing {
    /// The store among `files`, every file below the output folder.
    pub(crate) fn of(files: &BTreeSet<FilePath>) -> Self {
        StoreListing {
            descriptors: files.iter().filter_map(version_descriptor).collect(),
            objects: files.iter().filter_map(object_name).collect(),
        }
    }

    /// The path of each version's descriptor, to be read.
    pub(crate) fn descriptors(&self) -> impl Iterator<Item = &FilePath> {
        self.descriptors.iter().map(|(_, path)| path)
    }

    /// What the store holds, from `texts`, the text of each descriptor in the order of
    /// `descriptors`; `None` when one of them is not a descriptor of the store's format, or
    /// when there are not as many texts as descriptors.
    pub(crate) fn published(self, texts: &[String]) -> Option<PublishedStore> {
        if texts.len() != self.descriptors.len() {
            return None;
        }
        let versions = self
            .descriptors
            .into_iter()
            .map(|(name, _)| name)
            .zip(texts.iter().map(String::as_str).map(published_features))
            .map(lift)
            .collect::<Option<_>>()?;
        Some(PublishedStore {
            versions,
            objects: self.objects,
        })
    }
}

fn lift<T>((name, found): (VersionName, Option<T>)) -> Option<(VersionName, T)> {
    Some((name, found?))
}

/// The features the descriptor `text` names; `None` when it is not a descriptor of the
/// store's format.
fn published_features(text: &str) -> Option<Vec<PublishedFeature>> {
    let data: PublishedData = serde_json::from_str(text).ok()?;
    (data.format == STORE_FORMAT).then(|| data.features.into_iter().map(published).collect())
}

fn published(feature: FeatureData) -> PublishedFeature {
    PublishedFeature {
        page: feature.page,
        object: feature.object,
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

#[cfg(test)]
mod tests;
