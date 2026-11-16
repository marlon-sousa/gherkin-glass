//! Policy: a page's search box, with the words the script fills in.

use askama::Template;

use super::node_rendering::{Render, rendered};
use crate::entities::SearchWords;
use crate::policies::SEARCH_PAGE;

/// The search box of one page.
pub(super) struct SearchBox<'a> {
    pub(super) root: &'a str,
    pub(super) words: &'a SearchWords,
}

#[derive(Template)]
#[template(path = "search_box.html")]
struct SearchBoxTemplate<'a> {
    search: &'a SearchBox<'a>,
    /// The page of search results, relative to the site's root.
    page: &'a str,
}

impl Render for SearchBox<'_> {
    fn render(&self) -> String {
        rendered(&SearchBoxTemplate {
            search: self,
            page: SEARCH_PAGE,
        })
    }
}
