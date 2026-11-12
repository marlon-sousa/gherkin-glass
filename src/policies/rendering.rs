//! Policy: rendering, each node of the site tree rendered into its piece of a page through one trait, beside its template, the site's files from them, and the versions page.

mod feature_block;
mod feature_page;
mod folder_page;
mod node_rendering;
mod sentences_page;
mod site_data;
mod site_files;
mod tag_pages;
mod theme_stylesheet;
mod versions_page;

pub(crate) use site_files::render_site;
pub(crate) use versions_page::versions_page_files;
