//! Entity: what a build made, which the next build under `--watch` starts from.

use std::collections::BTreeSet;
use std::sync::Arc;

use crate::entities::{InputTree, RenderedSite, Settings, SiteTree, VersionName};

#[derive(Debug, Clone)]
pub(crate) struct BuildSnapshot {
    pub(crate) settings: Settings,
    /// Holds every entry of the input as the build last found it, also when the input stopped
    /// the build.
    pub(crate) input_tree: InputTree,
    /// The site tree of the last build that wrote its site, with the files rendered from it,
    /// shared with the snapshots made from this one that keep it; none while no build from
    /// nothing has got that far.
    pub(crate) site: Option<Arc<RenderedSite>>,
    /// The versions the output folder held when the build from nothing checked it.
    pub(crate) built_versions: BTreeSet<VersionName>,
}

impl BuildSnapshot {
    /// The site tree of the last build that wrote its site, which the converter starts from.
    pub(crate) fn site_tree(&self) -> Option<&SiteTree> {
        self.site.as_deref().map(|site| &site.tree)
    }
}
