//! Entity: what the converter makes of the input, the site tree and, beside it, the site's store.

use crate::entities::{SiteStore, SiteTree};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConvertedSite {
    pub(crate) site_tree: SiteTree,
    pub(crate) store: SiteStore,
}
