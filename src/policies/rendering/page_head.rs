//! Policy: a page's head, with its title, the themes' stylesheet and the links back to the shared assets: the stylesheet, the icon and the script.

use askama::Template;

use super::node_rendering::{Render, rendered};

/// The head of one page.
pub(super) struct PageHead<'a> {
    pub(super) title: &'a str,
    pub(super) root: &'a str,
    /// Already rendered from the site's accents.
    pub(super) stylesheet: &'a str,
}

#[derive(Template)]
#[template(path = "page_head.html")]
struct HeadTemplate<'a> {
    head: &'a PageHead<'a>,
}

impl Render for PageHead<'_> {
    fn render(&self) -> String {
        rendered(&HeadTemplate { head: self })
    }
}
