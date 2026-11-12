//! Entity: a folder page of the site tree, with the links to its subfolders' pages and its features' entries below it, as the folder tree and the rendering read them.

use ego_tree::NodeRef;

use super::SiteNode;
use crate::entities::{FeatureEntry, FolderLink, FolderPage};

/// A folder page node of the site tree.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SiteFolderPage<'a> {
    page: &'a FolderPage,
    node: NodeRef<'a, SiteNode>,
}

impl<'a> SiteFolderPage<'a> {
    /// `node` as a folder page, when it is one.
    pub(super) fn of(node: NodeRef<'a, SiteNode>) -> Option<SiteFolderPage<'a>> {
        match node.value() {
            SiteNode::FolderPage(page) => Some(SiteFolderPage { page, node }),
            _ => None,
        }
    }

    pub(crate) fn page(&self) -> &'a FolderPage {
        self.page
    }

    /// The links to the subfolders' pages, in the order the page lists them.
    pub(crate) fn folders(&self) -> impl Iterator<Item = &'a FolderLink> + use<'a> {
        self.node
            .children()
            .filter_map(|child| match child.value() {
                SiteNode::SubfolderLink(link) => Some(link),
                _ => None,
            })
    }

    /// The features' entries, in the order the page lists them.
    pub(crate) fn features(&self) -> impl Iterator<Item = &'a FeatureEntry> + use<'a> {
        self.node
            .children()
            .filter_map(|child| match child.value() {
                SiteNode::FeatureEntry(entry) => Some(entry),
                _ => None,
            })
    }
}
