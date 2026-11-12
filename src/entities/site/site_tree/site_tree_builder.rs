//! Entity: the site tree's builder, whose typed adds are the only way to put a node in the site tree.

use ego_tree::Tree;

use super::{SiteNode, SiteTree};
use crate::entities::{
    FolderPage, Page, PageFrame, SearchDocument, SentencesPage, TagIndex, TagPage, TreeItem,
};

/// A site tree being built, each add putting its files after those already added.
#[derive(Debug)]
pub(crate) struct SiteTreeBuilder {
    tree: Tree<SiteNode>,
}

impl SiteTreeBuilder {
    /// A site tree with only the site, whose pages share `frame`.
    pub(crate) fn new(frame: PageFrame) -> SiteTreeBuilder {
        SiteTreeBuilder {
            tree: Tree::new(SiteNode::Site(frame)),
        }
    }

    pub(crate) fn feature_pages(self, pages: impl IntoIterator<Item = Page>) -> SiteTreeBuilder {
        self.files(pages.into_iter().map(SiteNode::FeaturePage))
    }

    pub(crate) fn folder_pages(
        self,
        pages: impl IntoIterator<Item = FolderPage>,
    ) -> SiteTreeBuilder {
        self.files(pages.into_iter().map(SiteNode::FolderPage))
    }

    pub(crate) fn sentences_page(self, page: SentencesPage) -> SiteTreeBuilder {
        self.files([SiteNode::SentencesPage(page)])
    }

    pub(crate) fn folder_tree(self, items: Vec<TreeItem>) -> SiteTreeBuilder {
        self.files([SiteNode::FolderTree(items)])
    }

    pub(crate) fn tag_index(self, index: TagIndex) -> SiteTreeBuilder {
        self.files([SiteNode::TagIndex(index)])
    }

    pub(crate) fn tag_pages(self, pages: impl IntoIterator<Item = TagPage>) -> SiteTreeBuilder {
        self.files(pages.into_iter().map(SiteNode::TagPage))
    }

    pub(crate) fn search_documents(self, documents: Vec<SearchDocument>) -> SiteTreeBuilder {
        self.files([SiteNode::SearchDocuments(documents)])
    }

    /// The assets named `names` in their folder.
    pub(crate) fn assets(self, names: impl IntoIterator<Item = String>) -> SiteTreeBuilder {
        self.files(names.into_iter().map(SiteNode::Asset))
    }

    pub(crate) fn build(self) -> SiteTree {
        SiteTree { tree: self.tree }
    }

    /// `nodes`, put below the site after the files already added.
    fn files(mut self, nodes: impl IntoIterator<Item = SiteNode>) -> SiteTreeBuilder {
        for node in nodes {
            self.tree.root_mut().append(node);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::PageOptions;
    use crate::policies::page_frame;

    fn a_tree_of_assets_and_views() -> SiteTree {
        SiteTreeBuilder::new(page_frame(&PageOptions::default()))
            .assets(["first.css".to_string()])
            .folder_tree(Vec::new())
            .search_documents(Vec::new())
            .assets(["last.js".to_string()])
            .build()
    }

    #[test]
    fn the_site_is_the_root_and_holds_every_file_in_the_order_added() {
        let tree = a_tree_of_assets_and_views();
        assert!(matches!(tree.tree.root().value(), SiteNode::Site(_)));
        assert_eq!(
            tree.files().cloned().collect::<Vec<_>>(),
            [
                SiteNode::Asset("first.css".to_string()),
                SiteNode::FolderTree(Vec::new()),
                SiteNode::SearchDocuments(Vec::new()),
                SiteNode::Asset("last.js".to_string()),
            ]
        );
    }

    #[test]
    fn a_file_holds_no_node_of_its_own() {
        let tree = a_tree_of_assets_and_views();
        assert!(!tree.tree.root().children().any(|file| file.has_children()));
    }

    #[test]
    fn the_frame_is_the_site_s() {
        let frame = page_frame(&PageOptions::default());
        assert_eq!(a_tree_of_assets_and_views().frame(), &frame);
    }
}
