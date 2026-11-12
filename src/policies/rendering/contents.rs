//! Policy: the contents of a feature's page, a link to each of its blocks, with a rule's scenarios inside it.

use askama::Template;

use super::node_rendering::{Render, rendered};
use crate::entities::ContentsEntry;

/// The contents of the page at `page`, from the page that shows them.
pub(super) struct Contents<'a> {
    pub(super) entries: &'a [ContentsEntry],
    /// Empty on the feature's own page.
    pub(super) page: String,
}

#[derive(Template)]
#[template(path = "contents.html")]
struct ContentsTemplate<'a> {
    contents: &'a Contents<'a>,
}

impl Render for Contents<'_> {
    fn render(&self) -> String {
        rendered(&ContentsTemplate { contents: self })
    }
}

/// The contents on the feature's own page.
impl Render for [ContentsEntry] {
    fn render(&self) -> String {
        Contents {
            entries: self,
            page: String::new(),
        }
        .render()
    }
}
