//! Entity: a folder of the site tree being built, the input folder or one inside it, as the site stage's policies read it: its names from the input folder in, its subfolders and its pages.

use ego_tree::{NodeId, NodeRef};

use super::{SiteFeaturePage, SiteFolderPage, SiteNode, SiteTreeNode};

/// Which folder of the site tree being built a page goes in; only the builder
/// that handed it out can use it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FolderId(pub(super) NodeId);

/// A folder node of the site tree being built, with everything below it.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SiteFolder<'a> {
    /// A folder node, the input folder or a folder inside it.
    pub(super) node: NodeRef<'a, SiteNode>,
}

impl<'a> SiteFolder<'a> {
    pub(crate) fn id(&self) -> FolderId {
        FolderId(self.node.id())
    }

    /// The folder's own name, or `None` for the input folder.
    pub(crate) fn name(&self) -> Option<&'a str> {
        folder_name(self.node.value())
    }

    /// The names of the folders from the input folder in, down to this one,
    /// the input folder left out: empty for the input folder.
    pub(crate) fn names(&self) -> Vec<String> {
        SiteTreeNode::new(self.node)
            .folders()
            .into_iter()
            .chain(self.name())
            .map(str::to_string)
            .collect()
    }

    /// The folders directly inside this one, in the order they were made.
    pub(crate) fn subfolders(&self) -> impl Iterator<Item = SiteFolder<'a>> + use<'a> {
        self.node.children().filter(is_folder).map(as_site_folder)
    }

    /// The folder directly inside this one named `name`, if there is one.
    pub(crate) fn subfolder(&self, name: &str) -> Option<SiteFolder<'a>> {
        self.subfolders().find(|subfolder| subfolder.is_named(name))
    }

    /// The pages of the features directly in this folder, in the order they were put in it.
    pub(crate) fn feature_pages(&self) -> impl Iterator<Item = SiteFeaturePage<'a>> + use<'a> {
        self.node.children().filter_map(SiteFeaturePage::of)
    }

    /// The pages of the features in this folder and every folder inside it, at any depth.
    pub(crate) fn every_feature_page(&self) -> impl Iterator<Item = SiteFeaturePage<'a>> + use<'a> {
        self.node.descendants().filter_map(SiteFeaturePage::of)
    }

    /// The folders inside this one, at any depth, this one left out.
    pub(crate) fn every_subfolder(&self) -> impl Iterator<Item = SiteFolder<'a>> + use<'a> {
        self.node
            .descendants()
            .skip(1)
            .filter(is_folder)
            .map(as_site_folder)
    }

    fn is_named(&self, name: &str) -> bool {
        self.name() == Some(name)
    }

    /// The folder's own page, once it is put in.
    pub(crate) fn folder_page(&self) -> Option<SiteFolderPage<'a>> {
        self.node.children().find_map(SiteFolderPage::of)
    }
}

fn as_site_folder(node: NodeRef<'_, SiteNode>) -> SiteFolder<'_> {
    SiteFolder { node }
}

/// Whether `node` is a folder inside the input folder.
fn is_folder(node: &NodeRef<'_, SiteNode>) -> bool {
    folder_name(node.value()).is_some()
}

/// The name of `node` when it is a folder inside the input folder.
pub(super) fn folder_name(node: &SiteNode) -> Option<&str> {
    match node {
        SiteNode::Folder(name) => Some(name),
        _ => None,
    }
}
