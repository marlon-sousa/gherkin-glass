//! Entity: the files of the store a build renders, and how the site writer writes them beside what the output folder's store already holds.

use std::collections::BTreeSet;

use crate::entities::{FilePath, PublishedFeature, SiteFile};

/// Every address is below the output folder, such as `data/objects/<name>.json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StoreFiles {
    /// One for each distinct object.
    pub(crate) objects: Vec<SiteFile>,
    pub(crate) descriptor: SiteFile,
    /// What the descriptor names, in its order.
    pub(crate) features: Vec<PublishedFeature>,
}

/// Every path is below the output folder, such as `data/objects/<name>.json`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct StorePlan {
    /// The files to write, each address below the output folder.
    pub(crate) writes: Vec<SiteFile>,
    /// What nothing references once the files are written, to remove after them.
    pub(crate) removes: BTreeSet<FilePath>,
    /// Every file of the store once the plan is followed.
    pub(crate) files: BTreeSet<FilePath>,
}
