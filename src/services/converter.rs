//! Service: the converter, turning the parsed features into the site tree.

use crate::entities::{ParsedFeature, Settings, SiteTree};
use crate::policies::site_tree;

/// What the controller asks of the converter.
pub(crate) trait ConverterApi: Send + Sync {
    /// The tree of the site, pages first, in file order.
    fn convert(&self, features: &[ParsedFeature], settings: &Settings) -> SiteTree;
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Converter;

impl ConverterApi for Converter {
    fn convert(&self, features: &[ParsedFeature], settings: &Settings) -> SiteTree {
        site_tree(features, &settings.pages)
    }
}
