//! Entity: a folder's page, the home page among them, every text in it as written or worded: its nodes and its content as the site stage builds it, with what is below its node.

use crate::entities::{ContentsEntry, LinkAddress, SiteAddress, Trail};

/// One folder page of the site, which holds the link to each of its
/// subfolders' pages and the entry of each of its features. Every `String`
/// is text, escaped when it is rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FolderPage {
    /// Always ending in `index.html`.
    pub(crate) address: SiteAddress,
    /// The `lang` of the page, that of gherkin-glass's own text.
    pub(crate) language: String,
    pub(crate) title: String,
    /// The folder's name, or the site's title on the home page.
    pub(crate) heading: String,
    pub(crate) trail: Trail,
    /// Only the home page's.
    pub(crate) totals: Vec<String>,
    /// Only the home page's: the link to the index of every tag.
    pub(crate) tags: Option<FolderLink>,
    /// Only the home page's: its link to the page of step sentences.
    pub(crate) sentences: Option<FolderLink>,
    pub(crate) words: FolderWords,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FolderLink {
    /// Relative to the folder page that links it.
    pub(crate) address: LinkAddress,
    pub(crate) name: String,
}

/// A feature file of the folder, as its page lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FeatureEntry {
    /// Relative to the folder page that lists it.
    pub(crate) address: LinkAddress,
    /// The `lang` of the feature's page.
    pub(crate) language: String,
    pub(crate) heading: String,
    /// The count of its scenarios, in words.
    pub(crate) scenarios: String,
    /// The button's accessible name, the count and the feature's heading.
    pub(crate) label: String,
    /// The contents of the feature's page, whose anchors are on that page.
    pub(crate) contents: Vec<ContentsEntry>,
}

/// gherkin-glass's own words on a folder page, from the interface text catalogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FolderWords {
    pub(crate) folders: String,
    pub(crate) features: String,
}

/// A folder's page with the links to its subfolders' pages and its features'
/// entries, each sorted, as the site stage builds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FolderPageContent {
    pub(crate) page: FolderPage,
    pub(crate) folders: Vec<FolderLink>,
    pub(crate) features: Vec<FeatureEntry>,
}
