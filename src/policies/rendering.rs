//! Policy: the rendering stage, the site writer's: each node of the site tree rendered into its piece of a page through one trait, beside its template, the versions page and the page of search results among them, the front page from its Markdown template, the site's files from them, the files of the site's store, what the output folder's store already holds and the plan by which the store is written, and whether the output folder is safe to write.

mod breadcrumb;
mod contents;
mod description;
mod doc_string;
mod examples_list;
mod feature_block;
mod feature_page;
mod folder_page;
mod front_page;
mod heading_parts;
mod keyword_note;
mod node_rendering;
mod output_safety;
mod page_frame;
mod page_head;
mod page_heading;
mod page_layout;
mod published_store;
mod search_box;
mod search_page;
mod sentences_page;
mod site_data;
mod site_files;
mod steps;
mod store_files;
mod store_format;
mod store_plan;
mod table;
mod tag_pages;
mod tags;
mod theme_stylesheet;
mod versions_page;

pub(crate) use output_safety::{Moment, output_safety};
pub(crate) use published_store::StoreListing;
pub(crate) use site_files::render_site;
pub(crate) use store_files::{DescriptorSettings, render_store};
pub(crate) use store_plan::plan_store;

use store_format::{
    GHG_VERSION, JSON, OBJECTS, PAGES_INDEX, STORE_FORMAT, VERSIONS, VERSIONS_LIST,
    descriptor_path, json_bytes, object_path, sha256_hex,
};
