//! Entity: an entry of a feature's contents, a link to one of its rules or scenarios, with a rule's scenarios inside it, and a tag's link to another site.

use crate::entities::HeadingPart;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContentsEntry {
    /// Percent-encoded, as an address holds it.
    pub(crate) anchor: String,
    pub(crate) heading: Vec<HeadingPart>,
    /// A rule's scenarios.
    pub(crate) entries: Vec<ContentsEntry>,
}

/// A tag's link to another site, from the configuration's `tag_links`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OutsideLink {
    pub(crate) address: String,
    pub(crate) text: String,
}
