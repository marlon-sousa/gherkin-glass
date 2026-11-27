//! Entity: the versions page, above every version's site, every text in it as written or worded, with the entries of its lists as the site stage builds them.

use crate::entities::VersionName;

/// Every `String` is text, escaped when it is rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VersionsPage {
    /// The `lang` of the page, that of gherkin-glass's own text.
    pub(crate) language: String,
    /// Its `h1` without a template.
    pub(crate) heading: String,
    /// Its title without a template: the heading and the site's title.
    pub(crate) title: String,
    pub(crate) words: VersionsWords,
}

/// A version in a list: its name, which is its folder's, and the words of its link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VersionEntry {
    pub(crate) name: VersionName,
    pub(crate) words: String,
}

/// The headings of the versions page's lists, from the interface text catalogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VersionsWords {
    pub(crate) development: String,
    pub(crate) releases: String,
    pub(crate) pre_releases: String,
}

/// The page with the entries of each of its lists, each in its order, as the site stage builds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VersionsPageContent {
    pub(crate) page: VersionsPage,
    pub(crate) development: Vec<VersionEntry>,
    pub(crate) releases: Vec<VersionEntry>,
    pub(crate) pre_releases: Vec<VersionEntry>,
}
