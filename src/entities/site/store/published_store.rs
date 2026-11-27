//! Entity: what the output folder's store holds when a command starts: what each descriptor names, the objects there, and what the versions page was last written with.

use std::collections::{BTreeMap, BTreeSet};

use crate::entities::{Accents, BuildTarget};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct PublishedStore {
    /// The descriptor of the site, of each version and of each preview.
    pub(crate) descriptors: BTreeMap<BuildTarget, PublishedDescriptor>,
    /// The objects' names, without `.json`.
    pub(crate) objects: BTreeSet<String>,
    /// `None` without `data/versions.json`.
    pub(crate) versions_page: Option<VersionsPageRecord>,
}

/// What a descriptor names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PublishedDescriptor {
    /// The version of the ghg that wrote it, whose assets its shell links.
    pub(crate) ghg: String,
    /// In its order.
    pub(crate) features: Vec<PublishedFeature>,
}

/// A feature a descriptor names: its page's name and its object's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PublishedFeature {
    pub(crate) page: String,
    pub(crate) object: String,
}

/// What the versions page was last written with, as `data/versions.json` holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VersionsPageRecord {
    /// The version of the ghg that wrote it, whose assets it links.
    pub(crate) ghg: String,
    pub(crate) title: String,
    pub(crate) accents: Accents,
    /// The text of its template; `None` for ghg's own.
    pub(crate) front_page: Option<String>,
}
