//! Entity: a folder page of the site tree, with the links to its subfolders' pages and its features' entries below it, as the folder tree and the rendering read them.

use ego_tree::NodeRef;

use super::SiteNode;
use crate::entities::{FeatureEntry, FolderLink, FolderPage};

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
        self.node.children().filter_map(subfolder_link)
    }

    /// The features' entries, in the order the page lists them.
    pub(crate) fn features(&self) -> impl Iterator<Item = &'a FeatureEntry> + use<'a> {
        self.node.children().filter_map(feature_entry)
    }
}

fn subfolder_link(node: NodeRef<'_, SiteNode>) -> Option<&FolderLink> {
    match node.value() {
        SiteNode::SubfolderLink(link) => Some(link),
        _ => None,
    }
}

fn feature_entry(node: NodeRef<'_, SiteNode>) -> Option<&FeatureEntry> {
    match node.value() {
        SiteNode::FeatureEntry(entry) => Some(entry),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::entities::PageOptions;
    use crate::test_support::site_of;

    #[test]
    fn a_folder_page_lists_its_subfolders_then_its_features() {
        let site = site_of(
            &[
                ("billing/refunds.feature", "Feature: Refunds\n"),
                ("billing/archive/old.feature", "Feature: Old\n"),
            ],
            &PageOptions::default(),
        );
        let Some(billing) = site
            .files()
            .filter_map(|file| file.folder_page())
            .find(|folder| folder.page().heading == "billing")
        else {
            panic!("billing's page");
        };
        let folders: Vec<_> = billing.folders().map(|link| link.name.as_str()).collect();
        assert_eq!(folders, ["archive"]);
        let features: Vec<_> = billing
            .features()
            .map(|entry| entry.address.to_string())
            .collect();
        assert_eq!(features, ["refunds.html"]);
    }
}
