//! Entity: the site tree's builder, whose typed adds are the only way to put a node in the site tree, each kind under the kind it belongs to: the versions, the versions page and its assets under the site, the folders under the input folder, each page in its folder, a feature page's blocks in it, a rule's background and scenarios in the rule, an outline's examples in the outline, each step in its block, a step's table or doc string in the step, and the version's other files under the version.

mod feature_page_nodes;

use ego_tree::{NodeId, NodeMut, NodeRef, Tree};

use feature_page_nodes::{copy_feature_page, place_feature_page};

use super::{FolderId, SiteFeaturePage, SiteFolder, SiteNode, SiteTree, SiteVersion};
use crate::entities::{
    FolderPageContent, GroupContent, PageContent, PageFrame, SearchDocument, SentenceContent,
    SentencesPageContent, TagFeatureContent, TagPagesContent, TreeItem, VersionName, VersionsList,
    VersionsPageContent,
};

/// A site tree being built, each add putting its nodes after those already
/// in the node it adds to.
#[derive(Debug)]
pub(crate) struct SiteTreeBuilder {
    tree: Tree<SiteNode>,
    version: NodeId,
    input_folder: NodeId,
    /// Every feature page, in the order it was put in.
    feature_pages: Vec<NodeId>,
}

impl SiteTreeBuilder {
    /// A site tree with only the site, whose pages share `frame`, the version
    /// it makes, `version`, and the version's input folder.
    pub(crate) fn new(frame: PageFrame, version: SiteVersion) -> SiteTreeBuilder {
        let mut tree = Tree::new(SiteNode::Site(frame));
        let mut site = tree.root_mut();
        let mut version_node = site.append(SiteNode::Version(version));
        let input_folder = version_node.append(SiteNode::InputFolder).id();
        let version = version_node.id();
        SiteTreeBuilder {
            tree,
            version,
            input_folder,
            feature_pages: Vec::new(),
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

    /// Each page put in the folder it is paired with, its blocks below it.
    pub(crate) fn feature_pages(
        &mut self,
        pages: impl IntoIterator<Item = (FolderId, PageContent)>,
    ) {
        for (folder, content) in pages {
            let id = place_feature_page(&mut self.node_mut(folder.0), content);
            self.feature_pages.push(id);
        }
    }

    /// Each page of the last site tree put in the folder it is paired with, with its blocks, as it
    /// was there.
    pub(crate) fn kept_feature_pages<'a>(
        &mut self,
        pages: impl IntoIterator<Item = (FolderId, SiteFeaturePage<'a>)>,
    ) {
        for (folder, page) in pages {
            let id = copy_feature_page(&mut self.node_mut(folder.0), page.tree_node());
            self.feature_pages.push(id);
        }
    }

    /// Every feature page put in so far, with its blocks, in the order it was put in.
    pub(crate) fn every_feature_page(&self) -> impl Iterator<Item = SiteFeaturePage<'_>> {
        self.feature_pages
            .iter()
            .filter_map(|id| SiteFeaturePage::of(self.node(*id)))
    }

    /// Each folder's page put in the folder it is paired with, its links and
    /// entries below it.
    pub(crate) fn folder_pages(
        &mut self,
        pages: impl IntoIterator<Item = (FolderId, FolderPageContent)>,
    ) {
        for (folder, content) in pages {
            let mut folder = self.node_mut(folder.0);
            let mut page = folder.append(SiteNode::FolderPage(content.page));
            for link in content.folders {
                page.append(SiteNode::SubfolderLink(link));
            }
            for entry in content.features {
                page.append(SiteNode::FeatureEntry(entry));
            }
        }
    }

    /// The page of step sentences, its groups below it, each group's
    /// sentences below the group and each sentence's uses below the sentence.
    pub(crate) fn sentences_page(&mut self, content: SentencesPageContent) {
        let mut version = self.node_mut(self.version);
        let mut page = version.append(SiteNode::SentencesPage(content.page));
        for GroupContent { group, sentences } in content.groups {
            let mut group = page.append(SiteNode::SentencesGroup(group));
            for SentenceContent { sentence, uses } in sentences {
                let mut sentence = group.append(SiteNode::Sentence(sentence));
                for found in uses {
                    sentence.append(SiteNode::SentenceUse(found));
                }
            }
        }
    }

    pub(crate) fn folder_tree(&mut self, items: Vec<TreeItem>) {
        self.version_files([SiteNode::FolderTree(items)]);
    }

    /// The index, its entries below it, then each tag's page, its features
    /// below it, each feature's scenarios below the feature.
    pub(crate) fn tag_pages(&mut self, content: TagPagesContent) {
        let mut version = self.node_mut(self.version);
        let mut index = version.append(SiteNode::TagIndex(content.index.index));
        for entry in content.index.entries {
            index.append(SiteNode::TagIndexEntry(entry));
        }
        for page_content in content.pages {
            let mut page = version.append(SiteNode::TagPage(page_content.page));
            for TagFeatureContent { feature, scenarios } in page_content.features {
                let mut feature = page.append(SiteNode::TagFeature(feature));
                for scenario in scenarios {
                    feature.append(SiteNode::TagScenario(scenario));
                }
            }
        }
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

    /// The versions page, its lists below it, each list's entries below the list.
    pub(crate) fn versions_page(&mut self, content: VersionsPageContent) {
        let mut site = self.tree.root_mut();
        let mut page = site.append(SiteNode::VersionsPage(content.page));
        for (list, entries) in [
            (VersionsList::Development, content.development),
            (VersionsList::Releases, content.releases),
            (VersionsList::PreReleases, content.pre_releases),
        ] {
            let mut list = page.append(SiteNode::VersionsList(list));
            for entry in entries {
                list.append(SiteNode::VersionEntry(entry));
            }
        }
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
