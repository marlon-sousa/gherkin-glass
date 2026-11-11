//! Entity: the name of every tag's page, each tag as written with its `@`.

use std::collections::BTreeMap;

use crate::entities::SiteAddress;

/// The folder of the site that holds the tag pages and their index.
pub(crate) const TAGS_FOLDER: &str = "tags";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TagNames {
    names: BTreeMap<String, String>,
}

impl TagNames {
    pub(crate) fn insert(&mut self, tag: String, name: String) {
        self.names.insert(tag, name);
    }

    /// Such as `slow`; empty for a tag that was never named, which a build never shows.
    pub(crate) fn name(&self, tag: &str) -> &str {
        self.names.get(tag).map(String::as_str).unwrap_or_default()
    }

    /// Such as `tags/slow.html`.
    pub(crate) fn address(&self, tag: &str) -> SiteAddress {
        SiteAddress::of(&[TAGS_FOLDER], &format!("{}.html", self.name(tag)))
    }
}
