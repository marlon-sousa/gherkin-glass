//! Entity: the index of every tag and each tag's page, as their templates show them, their nodes and their content as the site stage builds it, with what is below each node.

use crate::entities::{LinkAddress, OutsideLink, SiteAddress, Trail};

/// The index of every tag. Every `String` is text, escaped when it is
/// rendered, and every link is relative to the page that holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TagIndex {
    /// In the folder of the tag pages.
    pub(crate) address: SiteAddress,
    /// The index's `lang`: the catalogue's language.
    pub(crate) language: String,
    pub(crate) trail: Trail,
    pub(crate) title: String,
    /// Shown in place of the entries when there are none.
    pub(crate) empty: String,
}

/// The tag's link, with the words before and after it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TagIndexEntry {
    pub(crate) address: LinkAddress,
    pub(crate) tag: String,
    pub(crate) before: String,
    pub(crate) after: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TagPage {
    /// Such as `tags/slow.html`.
    pub(crate) address: SiteAddress,
    /// The page's `lang`: the catalogue's language.
    pub(crate) language: String,
    pub(crate) trail: Trail,
    pub(crate) title: String,
    /// The title's words before and after the tag, which is isolated in the markup.
    pub(crate) heading_before: String,
    pub(crate) tag: String,
    pub(crate) heading_after: String,
    pub(crate) outside: Option<OutsideLink>,
    /// Shown under a feature none of whose scenarios carries the tag.
    pub(crate) no_scenario: String,
}

/// A feature that writes the tag, on its page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TagFeature {
    pub(crate) address: LinkAddress,
    /// The feature page's title.
    pub(crate) heading: String,
    /// The feature page's `lang`, when it is not the tag page's.
    pub(crate) language: Option<String>,
}

/// The scenario's link, with the words before and after it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TagScenario {
    pub(crate) address: LinkAddress,
    pub(crate) heading: String,
    pub(crate) before: String,
    pub(crate) after: String,
}

/// The index and every tag's page, in tag order, with what each holds, as the site stage builds them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TagPagesContent {
    pub(crate) index: TagIndexContent,
    pub(crate) pages: Vec<TagPageContent>,
}

/// The index with its entries, in tag order, as the site stage builds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TagIndexContent {
    pub(crate) index: TagIndex,
    pub(crate) entries: Vec<TagIndexEntry>,
}

/// A tag's page with each feature that writes the tag, in path order, as the site stage builds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TagPageContent {
    pub(crate) page: TagPage,
    pub(crate) features: Vec<TagFeatureContent>,
}

/// A feature that writes the tag, with its scenarios that carry it, in file order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TagFeatureContent {
    pub(crate) feature: TagFeature,
    pub(crate) scenarios: Vec<TagScenario>,
}
