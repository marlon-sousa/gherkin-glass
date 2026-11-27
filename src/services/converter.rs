//! Service: the converter, making the site's store from the features of the input tree.

use crate::entities::{InputTree, Settings, SiteStore};
use crate::policies::build_store;

/// What the controller asks of the converter.
pub(crate) trait ConverterApi: Send + Sync {
    /// The site's store, the object of each feature of `input_tree` parsed without an error,
    /// with what its descriptor holds, built with the settings of every page.
    fn convert(&self, input_tree: &InputTree, settings: &Settings) -> SiteStore;
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Converter;

impl ConverterApi for Converter {
    fn convert(&self, input_tree: &InputTree, settings: &Settings) -> SiteStore {
        let parsed: Vec<_> = input_tree.features().collect();
        build_store(&parsed, &settings.pages)
    }
}
