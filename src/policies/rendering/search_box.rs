//! Policy: a page's search box, with the words the script fills in.

use askama::Template;

use super::node_rendering::{Render, rendered};
use crate::entities::SearchWords;

/// The search box of one page.
pub(super) struct SearchBox<'a> {
    /// From the page back to the site's root, where the search documents are.
    pub(super) root: &'a str,
    pub(super) words: &'a SearchWords,
}

#[derive(Template)]
#[template(path = "search_box.html")]
struct SearchBoxTemplate<'a> {
    search: &'a SearchBox<'a>,
}

impl Render for SearchBox<'_> {
    fn render(&self) -> String {
        rendered(&SearchBoxTemplate { search: self })
    }
}
