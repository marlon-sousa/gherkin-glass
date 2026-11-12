//! Entity: a site tree with the files rendered from it, which the next build keeps the file of a node from when the node has not changed.

use crate::entities::{SiteFiles, SiteTree};

#[derive(Debug)]
pub(crate) struct RenderedSite {
    pub(crate) tree: SiteTree,
    /// Rendered from `tree`, each file in the order of its node.
    pub(crate) files: SiteFiles,
}
