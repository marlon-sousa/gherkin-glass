//! Entity: what the output folder's store holds when a build starts: what the descriptor of each version names, and the objects there.

use std::collections::{BTreeMap, BTreeSet};

use crate::entities::VersionName;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct PublishedStore {
    /// Each version's features, in its descriptor's order.
    pub(crate) versions: BTreeMap<VersionName, Vec<PublishedFeature>>,
    /// The objects' names, without `.json`.
    pub(crate) objects: BTreeSet<String>,
}

/// A feature a descriptor names: its page's name and its object's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PublishedFeature {
    pub(crate) page: String,
    pub(crate) object: String,
}
