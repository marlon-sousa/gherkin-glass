//! Service: the converter, turning the parsed features into every file of the site.

use crate::entities::{ParsedFeature, Settings, SiteFiles};
use crate::policies::site_files;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Converter;

impl Converter {
    /// Every file of the site, pages first, in file order.
    pub(crate) fn convert(&self, features: &[ParsedFeature], settings: &Settings) -> SiteFiles {
        site_files(features, &settings.pages)
    }
}
