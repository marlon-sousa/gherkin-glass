//! Entity: the tags, the names of their pages, the pages themselves and the links to other sites.

mod tag_link;
mod tag_names;
mod tag_page;

pub(crate) use tag_link::TagLink;
pub(crate) use tag_names::{TAGS_FOLDER, TagNames};
pub(crate) use tag_page::{
    TagFeature, TagFeatureContent, TagIndex, TagIndexContent, TagIndexEntry, TagPage,
    TagPageContent, TagPagesContent, TagScenario,
};
