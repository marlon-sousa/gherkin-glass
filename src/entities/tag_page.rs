//! Entity: the index of every tag and each tag's page, as their templates show them.

use crate::entities::{Breadcrumb, OutsideLink, PageFrame};

/// Every `String` is text, escaped when it is rendered, and every address
/// is relative to the page that holds it and percent-encoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TagIndex {
    pub(crate) frame: PageFrame,
    pub(crate) breadcrumb: Breadcrumb,
    pub(crate) title: String,
    pub(crate) entries: Vec<TagIndexEntry>,
    /// Shown in place of the entries when there are none.
    pub(crate) empty: String,
}

/// The tag's link, with the words before and after it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TagIndexEntry {
    pub(crate) address: String,
    pub(crate) tag: String,
    pub(crate) before: String,
    pub(crate) after: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TagPage {
    /// Relative to the site's root, such as `tags/slow.html`.
    pub(crate) address: String,
    pub(crate) frame: PageFrame,
    pub(crate) breadcrumb: Breadcrumb,
    pub(crate) title: String,
    /// The title's words before and after the tag, which is isolated in the markup.
    pub(crate) heading_before: String,
    pub(crate) tag: String,
    pub(crate) heading_after: String,
    pub(crate) outside: Option<OutsideLink>,
    pub(crate) features: Vec<TagFeature>,
    /// Shown under a feature none of whose scenarios carries the tag.
    pub(crate) no_scenario: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TagFeature {
    pub(crate) address: String,
    /// The feature page's title.
    pub(crate) heading: String,
    /// The feature page's `lang`, when it is not the tag page's.
    pub(crate) language: Option<String>,
    pub(crate) scenarios: Vec<TagScenario>,
}

/// The scenario's link, with the words before and after it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TagScenario {
    pub(crate) address: String,
    pub(crate) heading: String,
    pub(crate) before: String,
    pub(crate) after: String,
}
