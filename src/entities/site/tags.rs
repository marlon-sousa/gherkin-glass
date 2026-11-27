//! Entity: the tags, the names of their pages and the links to other sites.

mod tag_link;
mod tag_names;

pub(crate) use tag_link::TagLink;
pub(crate) use tag_names::{TAGS_FOLDER, TagNames};
