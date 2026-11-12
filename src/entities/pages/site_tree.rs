//! Entity: the site tree the converter builds, every file of a site as a node of its kind, before it is rendered.

use crate::entities::{
    FolderPage, Page, SearchDocument, SentencesPage, TagIndex, TagPage, TreeItem,
};

/// One file of the site, as the node of its kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FileNode {
    Feature(Page),
    Folder(FolderPage),
    Sentences(SentencesPage),
    /// The folder tree's data, which the pages' script builds the tree from.
    FolderTree(Vec<TreeItem>),
    TagIndex(TagIndex),
    Tag(TagPage),
    /// The documents search reads.
    SearchDocuments(Vec<SearchDocument>),
    /// One of the assets every site has, by its name in their folder.
    Asset(String),
}

/// Every file of a site, pages first, in file order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SiteTree {
    pub(crate) files: Vec<FileNode>,
}
