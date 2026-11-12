//! Entity: the site tree the converter builds, before it is rendered: an ego-tree whose root is the site, with the frame every page shares, and whose nodes below it are every file of the site, each as the node of its kind.

mod site_tree_builder;

pub(crate) use site_tree_builder::SiteTreeBuilder;

use ego_tree::Tree;

use crate::entities::{
    FolderPage, Page, PageFrame, SearchDocument, SentencesPage, TagIndex, TagPage, TreeItem,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SiteNode {
    /// The site, which holds the frame every page shares.
    Site(PageFrame),
    FeaturePage(Page),
    FolderPage(FolderPage),
    SentencesPage(SentencesPage),
    /// The folder tree's data, which the pages' script builds the tree from.
    FolderTree(Vec<TreeItem>),
    TagIndex(TagIndex),
    TagPage(TagPage),
    /// The documents search reads.
    SearchDocuments(Vec<SearchDocument>),
    /// One of the assets every site has, by its name in their folder.
    Asset(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SiteTree {
    tree: Tree<SiteNode>,
}

impl SiteTree {
    /// The frame every page shares.
    #[expect(
        clippy::expect_used,
        reason = "the site tree builder puts the site at the root"
    )]
    pub(crate) fn frame(&self) -> &PageFrame {
        site_frame(self.tree.root().value())
            .expect("the site tree builder puts the site at the root")
    }

    /// Every file of the site, in its order.
    pub(crate) fn files(&self) -> impl Iterator<Item = &SiteNode> {
        self.tree.root().children().map(|file| file.value())
    }
}

/// The frame of `node`, when it is the site.
fn site_frame(node: &SiteNode) -> Option<&PageFrame> {
    match node {
        SiteNode::Site(frame) => Some(frame),
        _ => None,
    }
}
