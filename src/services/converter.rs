//! Service: the converter, turning the features of the input tree into the site tree.

use std::collections::BTreeSet;

use crate::entities::{Destination, InputTree, ParsedFeature, Settings, SiteTree, VersionName};
use crate::policies::site_tree;

/// What the controller asks of the converter.
pub(crate) trait ConverterApi: Send + Sync {
    /// The tree of the site, from the features of `input_tree` parsed without an error, each
    /// feature's page in its folder, with the version the settings name when the site is built
    /// into a folder, beside `built_versions`, those the output folder already holds.
    fn convert(
        &self,
        input_tree: &InputTree,
        settings: &Settings,
        built_versions: &BTreeSet<VersionName>,
    ) -> SiteTree;
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Converter;

impl ConverterApi for Converter {
    fn convert(
        &self,
        input_tree: &InputTree,
        settings: &Settings,
        built_versions: &BTreeSet<VersionName>,
    ) -> SiteTree {
        let features: Vec<ParsedFeature> = input_tree.features().cloned().collect();
        site_tree(
            &features,
            &settings.pages,
            built_version(settings),
            built_versions,
        )
    }
}

/// The version the build makes: the one the settings name, unless the site
/// is built in memory, which has no versions.
fn built_version(settings: &Settings) -> Option<&VersionName> {
    match settings.destination {
        Destination::Folder => settings.version.as_ref(),
        Destination::Memory => None,
    }
}
