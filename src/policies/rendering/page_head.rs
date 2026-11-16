//! Policy: a page's head, with its title, the themes' stylesheet and the links back to the shared assets: the stylesheet, the icon and the script.

use askama::Template;

use super::node_rendering::{Render, rendered};
use crate::entities::SearchIndexing;

/// The head of one page.
pub(super) struct PageHead<'a> {
    pub(super) title: &'a str,
    pub(super) root: &'a str,
    /// Already rendered from the site's accents.
    pub(super) stylesheet: &'a str,
    /// Whether the page lets search engines index it.
    pub(super) indexing: SearchIndexing,
}

#[derive(Template)]
#[template(path = "page_head.html")]
struct HeadTemplate<'a> {
    head: &'a PageHead<'a>,
    /// What the page asks of search engines, `None` for nothing.
    robots: Option<&'static str>,
}

impl Render for PageHead<'_> {
    fn render(&self) -> String {
        rendered(&HeadTemplate {
            head: self,
            robots: match self.indexing {
                SearchIndexing::Refused => Some("noindex"),
                SearchIndexing::Allowed => None,
            },
        })
    }
}
