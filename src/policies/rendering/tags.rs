//! Policy: the tags of a feature or a block, each linked to its tag's page and, where the configuration says, to another site.

use askama::Template;

use super::node_rendering::{Render, rendered};
use crate::entities::{OutsideLink, TagView};

#[derive(Template)]
#[template(path = "tags.html")]
struct TagsTemplate<'a> {
    tags: &'a [TagView],
}

#[derive(Template)]
#[template(path = "outside_link.html")]
struct OutsideLinkTemplate<'a> {
    link: &'a OutsideLink,
}

impl Render for [TagView] {
    fn render(&self) -> String {
        rendered(&TagsTemplate { tags: self })
    }
}

impl Render for OutsideLink {
    fn render(&self) -> String {
        rendered(&OutsideLinkTemplate { link: self })
    }
}
