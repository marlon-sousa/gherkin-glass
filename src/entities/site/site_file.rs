//! Entity: one file of a site, where it goes and what it holds.

use std::sync::Arc;

use crate::entities::{FilePath, SiteAddress};

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

    /// Its address, as a path.
    pub(crate) fn path(&self) -> FilePath {
        self.address.as_str().to_string()
    }
}
