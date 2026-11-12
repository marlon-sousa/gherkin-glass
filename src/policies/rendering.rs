//! Policy: the rendering stage, the site writer's: each node of the site tree rendered into its piece of a page through one trait, beside its template, the versions page among them, the site's files from them, and whether the output folder is safe to write.

mod breadcrumb;
mod contents;
mod description;
mod doc_string;
mod feature_block;
mod feature_page;
mod folder_page;
mod from_background;
mod node_rendering;
mod output_safety;
mod page_frame;
mod page_head;
mod page_layout;
mod search_box;
mod sentences_page;
mod site_data;
mod site_files;
mod steps;
mod table;
mod tag_pages;
mod tags;
mod theme_stylesheet;
mod versions_page;

pub(crate) use output_safety::{Moment, output_safety};
pub(crate) use site_files::render_site;
