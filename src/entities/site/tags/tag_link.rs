//! Entity: a link from the tags a pattern matches to another site, as the configuration's `tag_links` gives it.

use regex_automata::meta::Regex;

/// Two links are the same when they are written the same.
#[derive(Debug, Clone)]
pub(crate) struct TagLink {
    /// As written in the configuration.
    pub(crate) source: String,
    /// `source`, matched against the whole tag only.
    pub(crate) pattern: Regex,
    /// With the replacement syntax of the `regex` crate.
    pub(crate) address: String,
    /// With the replacement syntax of the `regex` crate.
    pub(crate) text: String,
}

impl PartialEq for TagLink {
    fn eq(&self, other: &Self) -> bool {
        (&self.source, &self.address, &self.text) == (&other.source, &other.address, &other.text)
    }
}

impl Eq for TagLink {}
