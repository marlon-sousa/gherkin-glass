//! Entity: the site's pages and its files, what every page has, a folder's page, the breadcrumb and the folder tree, the addresses of the site and its links, and what a build that succeeded wrote.

mod breadcrumb;
mod built_site;
mod folder_page;
mod link_address;
mod page_frame;
mod page_options;
mod site_address;
mod site_assets;
mod site_file;
mod tree_item;

pub(crate) use breadcrumb::{Breadcrumb, Crumb};
pub(crate) use built_site::BuiltSite;
pub(crate) use folder_page::{FeatureEntry, FolderLink, FolderPage, FolderWords};
pub(crate) use link_address::{LinkAddress, percent_encoded};
pub(crate) use page_frame::PageFrame;
pub(crate) use page_options::PageOptions;
pub(crate) use site_address::SiteAddress;
pub(crate) use site_assets::{ASSETS, SiteAssets};
pub(crate) use site_file::{SiteFile, SiteFiles};
pub(crate) use tree_item::TreeItem;
