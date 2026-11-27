//! Policy: what a descriptor already in the output folder's store names, read from its JSON.

use serde::Deserialize;

use super::STORE_FORMAT;
use crate::entities::PublishedFeature;

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

/// The features the descriptor `text` names; `None` when it is not a descriptor of the
/// store's format.
pub(crate) fn published_features(text: &str) -> Option<Vec<PublishedFeature>> {
    let data: PublishedData = serde_json::from_str(text).ok()?;
    (data.format == STORE_FORMAT).then(|| data.features.into_iter().map(published).collect())
}

fn published(feature: FeatureData) -> PublishedFeature {
    PublishedFeature {
        page: feature.page,
        object: feature.object,
    }
}
