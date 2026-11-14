//! Policy: a page's head, with its title, the themes' stylesheet and the links back to the shared assets: the stylesheet, the icon and the script.

use askama::Template;

use super::node_rendering::{Render, rendered};
use crate::entities::VersionListing;

/// The head of one page.
pub(super) struct PageHead<'a> {
    pub(super) title: &'a str,
    pub(super) root: &'a str,
    /// Already rendered from the site's accents.
    pub(super) stylesheet: &'a str,
    /// Whether the page is a preview's, which search engines are asked not to index.
    pub(super) listing: VersionListing,
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
            robots: match self.listing {
                VersionListing::Preview => Some("noindex"),
                VersionListing::Listed => None,
            },
        })
    }
}
