//! Entity: the site tree's builder, whose typed adds are the only way to put a node in the site tree, each kind under the kind it belongs to: the versions, the versions page and its assets under the site, the folders under the input folder, each page in its folder, and the version's other files under the version.

use ego_tree::{NodeId, NodeMut, NodeRef, Tree};

use super::{FolderId, SiteFolder, SiteNode, SiteTree};
use crate::entities::{
    FolderPage, Page, PageFrame, SearchDocument, SentencesPage, TagIndex, TagPage, TreeItem,
    VersionName, VersionsPage,
};

/// A site tree being built, each add putting its nodes after those already
/// in the node it adds to.
#[derive(Debug)]
pub(crate) struct SiteTreeBuilder {
    tree: Tree<SiteNode>,
    version: NodeId,
    input_folder: NodeId,
}

impl SiteTreeBuilder {
    /// A site tree with only the site, whose pages share `frame`, the version
    /// it makes, named `version` unless the site has no versions, and the
    /// version's input folder.
    pub(crate) fn new(frame: PageFrame, version: Option<VersionName>) -> SiteTreeBuilder {
        let mut tree = Tree::new(SiteNode::Site(frame));
        let mut site = tree.root_mut();
        let mut version_node = site.append(SiteNode::Version(version));
        let input_folder = version_node.append(SiteNode::InputFolder).id();
        let version = version_node.id();
        SiteTreeBuilder {
            tree,
            version,
            input_folder,
        }
    }

    /// The folder at `names` inside the input folder, from the outermost,
    /// made with every folder above it that is not there yet; the input
    /// folder itself when `names` is empty.
    pub(crate) fn folder_at(&mut self, names: &[&str]) -> FolderId {
        FolderId(names.iter().fold(self.input_folder, |parent, name| {
            self.subfolder(parent, name)
        }))
    }

    /// The folder `folder`, as it holds its folders and pages so far.
    pub(crate) fn site_folder(&self, folder: FolderId) -> SiteFolder<'_> {
        SiteFolder {
            node: self.node(folder.0),
        }
    }

    /// Every folder, the input folder first, then each folder before the folders inside it.
    pub(crate) fn folders(&self) -> impl Iterator<Item = SiteFolder<'_>> {
        let input = self.input_folder();
        [input].into_iter().chain(input.every_subfolder())
    }

    /// The input folder, as it holds its folders and pages so far.
    pub(crate) fn input_folder(&self) -> SiteFolder<'_> {
        self.site_folder(FolderId(self.input_folder))
    }

    /// Each page put in the folder it is paired with.
    pub(crate) fn feature_pages(&mut self, pages: impl IntoIterator<Item = (FolderId, Page)>) {
        for (folder, page) in pages {
            self.node_mut(folder.0).append(SiteNode::FeaturePage(page));
        }
    }

    /// Each folder's page put in the folder it is paired with.
    pub(crate) fn folder_pages(&mut self, pages: impl IntoIterator<Item = (FolderId, FolderPage)>) {
        for (folder, page) in pages {
            self.node_mut(folder.0).append(SiteNode::FolderPage(page));
        }
    }

    pub(crate) fn sentences_page(&mut self, page: SentencesPage) {
        self.version_files([SiteNode::SentencesPage(page)]);
    }

    pub(crate) fn folder_tree(&mut self, items: Vec<TreeItem>) {
        self.version_files([SiteNode::FolderTree(items)]);
    }

    pub(crate) fn tag_index(&mut self, index: TagIndex) {
        self.version_files([SiteNode::TagIndex(index)]);
    }

    pub(crate) fn tag_pages(&mut self, pages: impl IntoIterator<Item = TagPage>) {
        self.version_files(pages.into_iter().map(SiteNode::TagPage));
    }

    pub(crate) fn search_documents(&mut self, documents: Vec<SearchDocument>) {
        self.version_files([SiteNode::SearchDocuments(documents)]);
    }

    /// The version's assets, named `names` in their folder.
    pub(crate) fn assets(&mut self, names: impl IntoIterator<Item = String>) {
        self.version_files(names.into_iter().map(SiteNode::Asset));
    }

    /// The versions the output folder already holds and the build keeps, by their names.
    pub(crate) fn kept_versions(&mut self, names: impl IntoIterator<Item = VersionName>) {
        self.site_nodes(names.into_iter().map(SiteNode::KeptVersion));
    }

    pub(crate) fn versions_page(&mut self, page: VersionsPage) {
        self.site_nodes([SiteNode::VersionsPage(page)]);
    }

    /// The assets the versions page shares with every page, named `names` in their folder.
    pub(crate) fn versions_page_assets(&mut self, names: impl IntoIterator<Item = String>) {
        self.site_nodes(names.into_iter().map(SiteNode::Asset));
    }

    pub(crate) fn build(self) -> SiteTree {
        SiteTree { tree: self.tree }
    }

    /// `nodes`, put in the version, in no folder of the input, after the nodes already there.
    fn version_files(&mut self, nodes: impl IntoIterator<Item = SiteNode>) {
        let mut version = self.node_mut(self.version);
        for node in nodes {
            version.append(node);
        }
    }

    /// `nodes`, put in the site, after the nodes already there.
    fn site_nodes(&mut self, nodes: impl IntoIterator<Item = SiteNode>) {
        let mut site = self.tree.root_mut();
        for node in nodes {
            site.append(node);
        }
    }

    /// The folder named `name` directly inside `parent`, made when it is not there yet.
    fn subfolder(&mut self, parent: NodeId, name: &str) -> NodeId {
        let found = SiteFolder {
            node: self.node(parent),
        }
        .subfolder(name)
        .map(|folder| folder.id().0);
        found.unwrap_or_else(|| {
            self.node_mut(parent)
                .append(SiteNode::Folder(name.to_string()))
                .id()
        })
    }

    #[expect(
        clippy::expect_used,
        reason = "every id the builder holds or hands out names a node of its tree"
    )]
    fn node(&self, id: NodeId) -> NodeRef<'_, SiteNode> {
        self.tree
            .get(id)
            .expect("every id the builder holds or hands out names a node of its tree")
    }

    #[expect(
        clippy::expect_used,
        reason = "every id the builder holds or hands out names a node of its tree"
    )]
    fn node_mut(&mut self, id: NodeId) -> NodeMut<'_, SiteNode> {
        self.tree
            .get_mut(id)
            .expect("every id the builder holds or hands out names a node of its tree")
    }
}

#[cfg(test)]
mod tests;
