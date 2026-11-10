//! Entity: a folder's page, the home page among them, every text in it as written or worded.

use crate::entities::{Breadcrumb, ContentsEntry, PageFrame};

/// One folder page of the site. Every `String` is text, escaped when it is rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FolderPage {
    /// Relative to the site's root, with `/` between folders, always ending in `index.html`.
    pub(crate) address: String,
    /// The `lang` of the page, that of gherkin-glass's own text.
    pub(crate) language: String,
    pub(crate) title: String,
    /// The folder's name, or the site's title on the home page.
    pub(crate) heading: String,
    pub(crate) frame: PageFrame,
    pub(crate) breadcrumb: Breadcrumb,
    /// Only the home page's.
    pub(crate) totals: Vec<String>,
    /// Only the home page's: the link to the index of every tag.
    pub(crate) tags: Option<FolderLink>,
    pub(crate) folders: Vec<FolderLink>,
    pub(crate) features: Vec<FeatureEntry>,
    pub(crate) words: FolderWords,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FolderLink {
    /// Relative to the folder page that links it.
    pub(crate) address: String,
    pub(crate) name: String,
}

/// A feature file of the folder, as its page lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FeatureEntry {
    /// Relative to the folder page that lists it.
    pub(crate) address: String,
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
