//! Entity: the folders of the site, each folder's page and the folder tree's items.

mod folder_page;
mod tree_item;

pub(crate) use folder_page::{FeatureEntry, FolderLink, FolderPage, FolderWords};
pub(crate) use tree_item::TreeItem;
