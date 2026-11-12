//! Entity: one file of a site, where it goes and what it holds, and every file of a site with what the build has to say of them.

use std::sync::Arc;

use crate::entities::{BuiltSite, SiteAddress};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SiteFile {
    pub(crate) address: SiteAddress,
    pub(crate) contents: Arc<[u8]>,
}

impl SiteFile {
    /// The file at `address` that holds `text`.
    pub(crate) fn text(address: SiteAddress, text: String) -> SiteFile {
        SiteFile {
            address,
            contents: Arc::from(text.into_bytes()),
        }
    }
}

/// Every file of a site, and what the build has to say of them.
pub(crate) struct SiteFiles {
    /// The version's files, or the whole site's when it has no versions.
    pub(crate) files: Vec<SiteFile>,
    /// The files beside the versions, at the output folder's root: the
    /// versions page and its assets, none for a site without versions.
    pub(crate) root_files: Vec<SiteFile>,
    pub(crate) built: BuiltSite,
}
