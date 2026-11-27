//! Policy: the contents of a feature's page, as the home page lists them: a link to each of its rules and scenarios, with a rule's scenarios inside it.

use askama::Template;

use super::heading_parts::RenderIn;
use super::node_rendering::{Render, rendered};
use crate::entities::ContentsEntry;

/// The contents of the page at `page`, from the page that shows them.
pub(super) struct Contents<'a> {
    pub(super) entries: &'a [ContentsEntry],
    /// The address of the feature's page.
    pub(super) page: String,
    /// The feature's language, which the home page is not in.
    pub(super) language: Option<&'a str>,
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
