//! Policy: the tags, their names and order, their pages, and their links to other sites.

mod tag_linking;
mod tag_naming;
mod tag_pages;

pub(super) use tag_linking::outside_link;
pub(super) use tag_naming::{name_tags, tag_index_address, tags_of};
pub(super) use tag_pages::{build_tag_index, build_tag_page};
