//! Policy: the site's pages and what every page has, their addresses and language, their frame and breadcrumb, the folder pages and the folder tree, the site's title, its tree and its files, the themes' colours and their stylesheet, and the HTML rendered from the templates.

mod breadcrumb_trail;
mod folder_pages;
mod folder_tree;
mod page_address;
mod page_frame;
mod page_language;
mod page_rendering;
mod palettes;
mod site_rendering;
mod site_title;
mod site_tree;
mod theme_stylesheet;

pub(crate) use breadcrumb_trail::breadcrumb;
pub(crate) use folder_pages::build_folder_pages;
pub(crate) use folder_tree::{build_folder_tree, render_folder_tree};
pub(crate) use page_address::{FOLDER_PAGE, page_address};
pub(crate) use page_frame::page_frame;
pub(crate) use page_language::page_language;
pub(crate) use page_rendering::{
    render_folder_page, render_page, render_sentences_page, render_tag_index, render_tag_page,
};
pub(crate) use palettes::palette;
pub(crate) use site_rendering::{assets, render_site};
pub(crate) use site_title::site_title;
pub(crate) use site_tree::site_tree;
pub(crate) use theme_stylesheet::theme_stylesheet;
