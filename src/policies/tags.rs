//! Policy: the tags, their names and order, their pages, and their links to other sites.

mod tag_linking;
mod tag_naming;
mod tag_pages;

pub(crate) use tag_linking::{outside_link, unknown_group, whole_tag_pattern};
pub(crate) use tag_naming::{name_tags, tag_index_address, tags_of};
pub(crate) use tag_pages::{build_tag_index, build_tag_page, tag_pages_language};
