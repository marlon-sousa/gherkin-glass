//! Entity: the site tree the converter builds, before it is rendered: an ego-tree whose root is the site, with the frame every page shares, then the version the build makes, the versions the output folder already holds and, for a site with versions, the versions page and its assets; then the version's folders, from the input folder in, with their pages, each feature page holding its blocks and their steps, and the version's files in no folder of the input, each node as the node of its kind.

mod site_feature_page;
mod site_folder;
mod site_folder_page;
mod site_tree_builder;
mod site_tree_node;

pub(crate) use site_feature_page::SiteFeaturePage;
pub(crate) use site_folder::{FolderId, SiteFolder};
pub(crate) use site_folder_page::SiteFolderPage;
pub(crate) use site_tree_builder::SiteTreeBuilder;
pub(crate) use site_tree_node::SiteTreeNode;

use ego_tree::{NodeRef, Tree};

use crate::entities::{
    BackgroundBlock, BlockHeading, DocStringView, ExamplesBlock, FeatureEntry, FolderLink,
    FolderPage, GroupView, Page, PageFrame, RuleBlock, ScenarioBlock, SearchDocument, SentenceView,
    SentencesPage, StepView, TableView, TagFeature, TagIndex, TagIndexEntry, TagPage, TagScenario,
    TreeItem, UseView, VersionEntry, VersionName, VersionsList, VersionsPage,
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
    /// One of the versions page's lists, which holds its versions' entries.
    VersionsList(VersionsList),
    /// A version's entry in a list of the versions page.
    VersionEntry(VersionEntry),
    /// The input folder, which holds the home page.
    InputFolder,
    /// A folder of the input, by its name, inside the input folder or another folder.
    Folder(String),
    /// A feature's page, which holds its blocks.
    FeaturePage(Page),
    /// A background, in its feature's page or its rule, which holds its steps.
    Background(BackgroundBlock),
    /// A rule, in its feature's page, which holds its background and its scenarios.
    Rule(RuleBlock),
    /// A scenario or an outline, in its feature's page or its rule, which
    /// holds its steps, then an outline's examples and examples as scenarios.
    Scenario(ScenarioBlock),
    /// An examples block of an outline.
    Examples(ExamplesBlock),
    /// An outline's examples as scenarios, by its heading, which holds the scenario of each row.
    ExamplesAsScenarios(BlockHeading),
    /// The scenario one row of an outline's examples makes, by its heading, which holds its steps.
    ExampleScenario(BlockHeading),
    /// A step of a background or a scenario, which holds its data table or its doc string.
    Step(StepView),
    /// A step's data table.
    StepTable(TableView),
    /// A step's doc string.
    DocString(DocStringView),
    /// A folder's page, which holds the links to its subfolders' pages and its features' entries.
    FolderPage(FolderPage),
    /// The link on a folder's page to a subfolder's page.
    SubfolderLink(FolderLink),
    /// A feature's entry on its folder's page.
    FeatureEntry(FeatureEntry),
    /// The page of step sentences, which holds a group for each kind of step.
    SentencesPage(SentencesPage),
    /// The sentences of one kind of step, which holds each of them.
    SentencesGroup(GroupView),
    /// A step sentence, which holds each of its uses.
    Sentence(SentenceView),
    /// A use of a step sentence.
    SentenceUse(UseView),
    /// The folder tree's data, which the pages' script builds the tree from.
    FolderTree(Vec<TreeItem>),
    /// The index of every tag, which holds an entry for each.
    TagIndex(TagIndex),
    /// A tag's entry in the index.
    TagIndexEntry(TagIndexEntry),
    /// A tag's page, which holds each feature that writes the tag.
    TagPage(TagPage),
    /// A feature on a tag's page, which holds each of its scenarios that carry the tag.
    TagFeature(TagFeature),
    /// A scenario on a tag's page.
    TagScenario(TagScenario),
    /// The documents search reads.
    SearchDocuments(Vec<SearchDocument>),
    /// One of the assets every site has, by its name in their folder.
    Asset(String),
}

impl SiteNode {
    /// Whether the node makes a file of the site, rather than standing for the
    /// site, a version or a folder, or being part of a page.
    fn is_file(&self) -> bool {
        matches!(
            self,
            SiteNode::VersionsPage(_)
                | SiteNode::FeaturePage(_)
                | SiteNode::FolderPage(_)
                | SiteNode::SentencesPage(_)
                | SiteNode::FolderTree(_)
                | SiteNode::TagIndex(_)
                | SiteNode::TagPage(_)
                | SiteNode::SearchDocuments(_)
                | SiteNode::Asset(_)
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
    pub(crate) fn files(&self) -> impl Iterator<Item = SiteTreeNode<'_>> {
        self.tree
            .root()
            .children()
            .filter(is_version)
            .flat_map(|version| version.descendants())
            .filter(is_file)
            .map(SiteTreeNode::new)
    }

    /// The files beside the versions, at the output folder's root: the
    /// versions page and its assets, none for a site without versions.
    pub(crate) fn root_files(&self) -> impl Iterator<Item = SiteTreeNode<'_>> {
        self.tree
            .root()
            .children()
            .filter(is_file)
            .map(SiteTreeNode::new)
    }
}

fn is_file(node: &NodeRef<'_, SiteNode>) -> bool {
    node.value().is_file()
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
