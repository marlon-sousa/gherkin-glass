//! Entity: the versions page, above every version's site, as its template shows it, every text in it as written or worded: its nodes and its content as the site stage builds it, with what is below each node.

use crate::entities::VersionName;

/// The page, which holds its three lists. Every `String` is text, escaped when it is rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VersionsPage {
    /// The `lang` of the page, that of gherkin-glass's own text.
    pub(crate) language: String,
    pub(crate) title: String,
    pub(crate) words: VersionsWords,
}

/// One of the versions page's lists, which holds its versions' entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VersionsList {
    Development,
    Releases,
    PreReleases,
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
