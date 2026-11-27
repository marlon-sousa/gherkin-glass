//! Entity: where a file is in the output folder, or in a site in memory, relative to its root.

use derive_more::Display;

/// The names of the folders the file is in, from the outermost, then the file's name, each as
/// written and joined by `/`.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
pub(crate) struct SiteAddress {
    address: String,
}

impl SiteAddress {
    /// The file at `path`, its folders and its name joined by `/`, below the root.
    pub(crate) fn at_root(path: &str) -> SiteAddress {
        SiteAddress {
            address: path.to_string(),
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.address
    }
}
