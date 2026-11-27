//! Policy: the files of the site's store a build renders, as compact JSON: each distinct object, named by the SHA-256 of its bytes, and the descriptor of the site or of the version.

mod descriptor_data;
mod object_data;

use std::collections::BTreeMap;
use std::sync::Arc;

pub(crate) use descriptor_data::DescriptorSettings;
use descriptor_data::descriptor_json;
use object_data::object_json;

use crate::entities::{
    DescriptorFeature, PublishedFeature, SiteAddress, SiteFile, SiteStore, StoreFiles,
};
use crate::policies::rendering::{
    PAGES_INDEX, VERSIONS_LIST, descriptor_path, object_path, sha256_hex,
};

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
    let indexes = match settings.version {
        Some(_) => vec![VERSIONS_LIST.to_string(), PAGES_INDEX.to_string()],
        None => Vec::new(),
    };
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
        indexes,
    }
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

#[cfg(test)]
mod tests;
