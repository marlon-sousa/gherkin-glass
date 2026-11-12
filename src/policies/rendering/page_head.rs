//! Policy: a page's head, with its title, the themes' stylesheet and the links back to the shared assets.

use askama::Template;

use super::node_rendering::{Render, rendered};
use crate::entities::Accents;

/// The head of one page.
pub(super) struct PageHead<'a> {
    pub(super) title: &'a str,
    /// From the page back to the site's root.
    pub(super) root: &'a str,
    pub(super) accents: &'a Accents,
}

#[derive(Template)]
#[template(path = "page_head.html")]
struct HeadTemplate<'a> {
    title: &'a str,
    root: &'a str,
    stylesheet: String,
}

impl Render for PageHead<'_> {
    fn render(&self) -> String {
        rendered(&HeadTemplate {
            title: self.title,
            root: self.root,
            stylesheet: self.accents.render(),
        })
    }
}
