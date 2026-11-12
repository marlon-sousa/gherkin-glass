//! Policy: the folders of the site, their pages and the folder tree built from them.

mod folder_pages;
mod folder_tree;

pub(super) use folder_pages::build_folder_pages;
pub(super) use folder_tree::build_folder_tree;
