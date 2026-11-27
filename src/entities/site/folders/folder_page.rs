//! Entity: the home page of a site without versions or of a preview, which its shell holds, every text in it as written or worded: its totals and links, and the folders and features at the top of its input.

use crate::entities::{ContentsEntry, HeadingPart, LinkAddress};

/// Every `String` is text, escaped when it is rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HomePage {
    /// The site's title, its `h1` and its page's title without a template.
    pub(crate) title: String,
    pub(crate) totals: Vec<String>,
    /// The link to the index of every tag.
    pub(crate) tags: FolderLink,
    /// The link to the page of step sentences.
    pub(crate) sentences: FolderLink,
    /// In a site of the files `--only` lists when none of them is read: the sentence that says
    /// so.
    pub(crate) nothing_given: Option<String>,
    pub(crate) words: FolderWords,
    /// Sorted by their names' character codes.
    pub(crate) folders: Vec<FolderLink>,
    /// Sorted by their files' names' character codes.
    pub(crate) features: Vec<FeatureEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FolderLink {
    pub(crate) address: LinkAddress,
    pub(crate) name: String,
}

/// A feature file at the top of the input, as the home page lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FeatureEntry {
    pub(crate) address: LinkAddress,
    /// The `lang` of the feature's page.
    pub(crate) language: String,
    pub(crate) heading: Vec<HeadingPart>,
    /// The count of its scenarios, in words.
    pub(crate) scenarios: String,
    /// The button's accessible name, the count and the feature's heading.
    pub(crate) label: String,
    /// The contents of the feature's page.
    pub(crate) contents: Vec<ContentsEntry>,
}

/// gherkin-glass's own words on the home page, from the interface text catalogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FolderWords {
    pub(crate) folders: String,
    pub(crate) features: String,
}
