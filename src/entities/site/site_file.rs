//! Entity: one file of a site, where it goes and what it holds, and every file of a site with what the build has to say of them.

use crate::entities::{BuiltSite, SiteAddress};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SiteFile {
    pub(crate) address: SiteAddress,
    pub(crate) contents: Vec<u8>,
}

impl SiteFile {
    /// The file at `address` that holds `text`.
    pub(crate) fn text(address: SiteAddress, text: String) -> SiteFile {
        SiteFile {
            address,
            contents: text.into_bytes(),
        }
    }
}

/// Every file of a site, and what the build has to say of them.
pub(crate) struct SiteFiles {
    pub(crate) files: Vec<SiteFile>,
    pub(crate) built: BuiltSite,
}
