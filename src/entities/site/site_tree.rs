//! Entity: the site tree the converter builds, before it is rendered: an ego-tree whose root is the site, with the frame every page shares, then the version the build makes, the versions the output folder already holds and, for a site with versions, the versions page and its assets; then the version's folders, from the input folder in, with their pages, and the version's files in no folder of the input, each node as the node of its kind.

mod site_folder;
mod site_tree_builder;

pub(crate) use site_folder::{FolderId, SiteFolder};
pub(crate) use site_tree_builder::SiteTreeBuilder;

use ego_tree::{NodeRef, Tree};

use crate::entities::{
    FolderPage, Page, PageFrame, SearchDocument, SentencesPage, TagIndex, TagPage, TreeItem,
    VersionName, VersionsPage,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SiteNode {
    /// The site, which holds the frame every page shares.
    Site(PageFrame),
    /// The version of the site the build makes, by its name, or `None` for a site without versions.
    Version(Option<VersionName>),
    /// A version the output folder already holds, which the build keeps, by its name.
    KeptVersion(VersionName),
    /// The page that lists every version, at the output folder's root.
    VersionsPage(VersionsPage),
    /// The input folder, which holds the home page.
    InputFolder,
    /// A folder of the input, by its name, inside the input folder or another folder.
    Folder(String),
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

impl SiteNode {
    /// Whether the node makes a file of the site, rather than standing for the site, a version or a folder.
    fn is_file(&self) -> bool {
        !matches!(
            self,
            SiteNode::Site(_)
                | SiteNode::Version(_)
                | SiteNode::KeptVersion(_)
                | SiteNode::InputFolder
                | SiteNode::Folder(_)
        )
    }
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

    /// Every file of the version the build makes, folder by folder from the input folder in, then those in no folder of the input.
    pub(crate) fn files(&self) -> impl Iterator<Item = &SiteNode> {
        self.tree
            .root()
            .children()
            .filter(is_version)
            .flat_map(|version| version.descendants())
            .map(|node| node.value())
            .filter(|node| node.is_file())
    }

    /// The files beside the versions, at the output folder's root: the
    /// versions page and its assets, none for a site without versions.
    pub(crate) fn root_files(&self) -> impl Iterator<Item = &SiteNode> {
        self.tree
            .root()
            .children()
            .map(|node| node.value())
            .filter(|node| node.is_file())
    }
}

fn is_version(node: &NodeRef<'_, SiteNode>) -> bool {
    matches!(node.value(), SiteNode::Version(_))
}

/// The frame of `node`, when it is the site.
fn site_frame(node: &SiteNode) -> Option<&PageFrame> {
    match node {
        SiteNode::Site(frame) => Some(frame),
        _ => None,
    }
}
