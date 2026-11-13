//! Policy: a page's `h1` and the content under it, rendered from their template, as a folder's page and ghg's own front page write them.

use askama::Template;

/// The `h1` of a page, then its content, already rendered, which starts with its own line
/// break when it is not empty.
#[derive(Template)]
#[template(path = "page_heading.html")]
pub(super) struct PageHeading<'a> {
    pub(super) heading: &'a str,
    pub(super) content: String,
}
