//! Policy: what a build that succeeded says of the site it stored: how many feature pages and scenarios it has, and whether it has no title of its own.

use crate::entities::{BuiltSite, PageOptions, SiteStore};

use super::block_counts::scenario_count;

/// What the build of `store`, with the settings of every page `options`, says of it; how large
/// its data is, the rendering stage says.
pub(crate) fn describe_site(store: &SiteStore, options: &PageOptions) -> BuiltSite {
    BuiltSite {
        large_site_data: None,
        untitled: options.title.is_none(),
        features: store.features.len(),
        scenarios: store
            .features
            .iter()
            .map(|feature| scenario_count(&feature.object))
            .sum(),
    }
}
