//! Entity: one file of a site, where it goes and what it holds.

use crate::entities::SiteAddress;

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
