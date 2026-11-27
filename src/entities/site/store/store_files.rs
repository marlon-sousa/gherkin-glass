//! Entity: the files of the store a build renders, and how a command writes the output folder: what it writes, in order, what it removes after them, and what the marker lists before and after.

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
    /// The size in bytes of the objects, only when it is above the size at which the pages and
    /// search can be slow to start.
    pub(crate) large_site_data: Option<usize>,
}

/// Every path is below the output folder.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct StorePlan {
    /// What the marker lists before anything is written: every file about to be there and
    /// every file still to be removed.
    pub(crate) first_marker: BTreeSet<FilePath>,
    /// In the order they are written.
    pub(crate) writes: Vec<SiteFile>,
    /// Files and folders, each descriptor before anything else.
    pub(crate) removes: Vec<FilePath>,
    /// What the marker lists once the plan is followed.
    pub(crate) last_marker: BTreeSet<FilePath>,
}
