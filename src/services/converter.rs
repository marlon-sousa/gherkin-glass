//! Service: the converter, making the site tree anew from the features of the input tree and the last site tree.

use std::collections::BTreeSet;

use crate::entities::{Destination, InputTree, Settings, SiteTree, VersionName};
use crate::policies::site_tree;

/// What the controller asks of the converter.
pub(crate) trait ConverterApi: Send + Sync {
    /// The tree of the site, from the features of `input_tree` parsed without an error, each
    /// feature's page in its folder, copied from `last`, the last site tree, when its file is
    /// unchanged and it links to the same tag pages, with the version the settings name when the
    /// site is built into a folder, beside `built_versions`, those the output folder already
    /// holds.
    fn convert(
        &self,
        input_tree: &InputTree,
        last: Option<&SiteTree>,
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
        last: Option<&SiteTree>,
        settings: &Settings,
        built_versions: &BTreeSet<VersionName>,
    ) -> SiteTree {
        let features: Vec<_> = input_tree.features().collect();
        site_tree(
            &features,
            last,
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
