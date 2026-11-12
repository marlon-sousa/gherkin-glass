//! Policy: the site's pages and what every page has, their addresses and language, their frame and breadcrumb, the folder pages and the folder tree, the site's title and its tree, and the themes' colours.

mod breadcrumb_trail;
mod folder_pages;
mod folder_tree;
mod page_address;
mod page_frame;
mod page_language;
mod palettes;
mod site_title;
mod site_tree;

pub(crate) use breadcrumb_trail::breadcrumb;
pub(crate) use folder_pages::build_folder_pages;
pub(crate) use folder_tree::build_folder_tree;
pub(crate) use page_address::{FOLDER_PAGE, page_address};
pub(crate) use page_frame::page_frame;
pub(crate) use page_language::page_language;
pub(crate) use palettes::palette;
pub(crate) use site_title::site_title;
pub(crate) use site_tree::site_tree;
