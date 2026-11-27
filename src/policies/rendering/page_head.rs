//! Policy: the head of a shell and of the versions page, with its title, its description, what it asks of search engines, the themes' stylesheet, the links to the shared assets, the stylesheet, the icon and the script, and the words the script draws pages with.

use askama::Template;

use super::node_rendering::{Render, rendered};

/// The head of one page.
pub(super) struct PageHead<'a> {
    pub(super) title: &'a str,
    /// A shell's; `None` for the versions page.
    pub(super) description: Option<&'a str>,
    /// Whether it asks search engines not to index the page.
    pub(super) unindexed: bool,
    /// The folder of the shared assets, relative to the page, such as `../assets/2.0.0`.
    pub(super) assets: &'a str,
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
