//! Service: the converter, turning the parsed features into every file of the site.

use crate::entities::{ParsedFeature, Settings, SiteFiles};
use crate::policies::site_files;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Converter;

impl Converter {
    /// Every file of the site, pages first, in file order. It cannot fail:
    /// every clash is found while the input is parsed.
    pub(crate) fn convert(&self, features: &[ParsedFeature], settings: &Settings) -> SiteFiles {
        site_files(features, &settings.pages)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::entities::{Destination, PageOptions, Unnamed};

    #[test]
    fn an_empty_input_still_has_its_home_page() {
        let settings = Settings {
            input: PathBuf::from("in"),
            output: PathBuf::from("site"),
            destination: Destination::Folder,
            unnamed: Unnamed::Allowed,
            pages: PageOptions::default(),
            version: None,
            replace: true,
        };
        let site = Converter.convert(&[], &settings);
        assert!(
            site.files
                .iter()
                .any(|file| file.address.as_str() == "index.html")
        );
    }
}
