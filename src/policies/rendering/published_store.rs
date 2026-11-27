//! Policy: what the output folder's store already holds: each descriptor, by the paths of the folder's files, what each names and the ghg that wrote it, read from its JSON, the objects there, and what the versions page was last written with, read from the list of versions.

use std::collections::BTreeSet;

use serde::Deserialize;

use crate::entities::{
    Accents, BuildTarget, Colour, FilePath, PublishedDescriptor, PublishedFeature, PublishedStore,
    VersionName, VersionsPageRecord,
};
use crate::policies::rendering::{
    JSON, OBJECTS, PREVIEWS, SITE_DESCRIPTOR, STORE_FORMAT, VERSIONS, VERSIONS_LIST,
};

/// The store among the files of an output folder: each descriptor, the name of each object,
/// and whether it holds the list of versions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StoreListing {
    descriptors: Vec<(BuildTarget, FilePath)>,
    objects: BTreeSet<String>,
    versions_list: bool,
}

/// A descriptor of the store, but for the members the site writer does not read.
#[derive(Deserialize)]
struct PublishedData {
    format: u32,
    ghg: String,
    features: Vec<FeatureData>,
}

#[derive(Deserialize)]
struct FeatureData {
    page: String,
    object: String,
}

/// The list of versions, but for the members the site writer does not read.
#[derive(Deserialize)]
struct VersionsData {
    format: u32,
    versions_page: VersionsPageData,
}

#[derive(Deserialize)]
struct VersionsPageData {
    ghg: String,
    title: String,
    accent_light: String,
    accent_dark: String,
    front_page: Option<String>,
}

impl StoreListing {
    /// The store among `files`, every file below the output folder.
    pub(crate) fn of(files: &BTreeSet<FilePath>) -> Self {
        StoreListing {
            descriptors: files.iter().filter_map(descriptor_of).collect(),
            objects: files.iter().filter_map(object_name).collect(),
            versions_list: files.contains(VERSIONS_LIST),
        }
    }

    /// The path of each descriptor, to be read, then of the list of versions, when there is one.
    pub(crate) fn to_read(&self) -> impl Iterator<Item = &str> {
        self.descriptors
            .iter()
            .map(|(_, path)| path.as_str())
            .chain(self.versions_list.then_some(VERSIONS_LIST))
    }

    /// What the store holds, from `texts`, the text of each file of `to_read`, in its order;
    /// `None` when one of them is not of the store's format, or when there are not as many texts
    /// as files.
    pub(crate) fn published(self, texts: &[String]) -> Option<PublishedStore> {
        let expected = self.descriptors.len() + usize::from(self.versions_list);
        if texts.len() != expected {
            return None;
        }
        let (descriptor_texts, list_text) = texts.split_at(self.descriptors.len());
        let descriptors = self
            .descriptors
            .into_iter()
            .map(|(target, _)| target)
            .zip(
                descriptor_texts
                    .iter()
                    .map(|text| published_descriptor(text)),
            )
            .map(lift)
            .collect::<Option<_>>()?;
        let versions_page = match list_text.first() {
            Some(text) => Some(versions_page(text)?),
            None => None,
        };
        Some(PublishedStore {
            descriptors,
            objects: self.objects,
            versions_page,
        })
    }
}

fn lift<T>((target, found): (BuildTarget, Option<T>)) -> Option<(BuildTarget, T)> {
    Some((target, found?))
}

/// What the descriptor `text` names; `None` when it is not a descriptor of the store's format.
fn published_descriptor(text: &str) -> Option<PublishedDescriptor> {
    let data: PublishedData = serde_json::from_str(text).ok()?;
    (data.format == STORE_FORMAT).then(|| PublishedDescriptor {
        ghg: data.ghg,
        features: data.features.into_iter().map(published).collect(),
    })
}

/// What the versions page was last written with, as the list of versions `text` holds it;
/// `None` when it is not one of the store's format.
fn versions_page(text: &str) -> Option<VersionsPageRecord> {
    let data: VersionsData = serde_json::from_str(text).ok()?;
    let page = data.versions_page;
    (data.format == STORE_FORMAT).then_some(())?;
    Some(VersionsPageRecord {
        ghg: page.ghg,
        title: page.title,
        accents: Accents {
            light: Colour::from_hex(&page.accent_light)?,
            dark: Colour::from_hex(&page.accent_dark)?,
        },
        front_page: page.front_page,
    })
}

fn published(feature: FeatureData) -> PublishedFeature {
    PublishedFeature {
        page: feature.page,
        object: feature.object,
    }
}

/// The target whose descriptor is at `path`, when it is one.
fn descriptor_of(path: &FilePath) -> Option<(BuildTarget, FilePath)> {
    if path == SITE_DESCRIPTOR {
        return Some((BuildTarget::Site, path.clone()));
    }
    let named = |folder: &str| {
        path.strip_prefix(folder)?
            .strip_suffix(JSON)
            .and_then(VersionName::new)
    };
    let target = named(VERSIONS)
        .map(BuildTarget::Version)
        .or_else(|| named(PREVIEWS).map(BuildTarget::Preview))?;
    Some((target, path.clone()))
}

/// The name of the object at `path`, when it is one.
fn object_name(path: &FilePath) -> Option<String> {
    let name = path.strip_prefix(OBJECTS)?.strip_suffix(JSON)?;
    (!name.contains('/')).then(|| name.to_string())
}

#[cfg(test)]
mod tests;
