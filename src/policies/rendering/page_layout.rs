//! Policy: a page of the site, its head, frame, breadcrumb and search box around its main content, each rendered as its own node.

use askama::Template;

use super::node_rendering::{Render, page};
use super::page_frame::TreeLandmark;
use super::page_head::PageHead;
use super::search_box::SearchBox;
use crate::entities::{Breadcrumb, PageFrame};

/// What a page has around its main content.
pub(super) struct PageAround<'a> {
    /// The `lang` of the page.
    pub(super) language: &'a str,
    pub(super) title: &'a str,
    /// From the page back to the site's root.
    pub(super) root: String,
    pub(super) frame: &'a PageFrame,
    pub(super) breadcrumb: &'a Breadcrumb,
}

#[derive(Template)]
#[template(path = "page_layout.html")]
struct LayoutTemplate<'a> {
    language: &'a str,
    head: String,
    frame: String,
    tree: String,
    breadcrumb: String,
    search: String,
    main: String,
}

/// The whole page whose content, already rendered, is `main`.
pub(super) fn whole_page(around: &PageAround<'_>, main: String) -> String {
    page(&LayoutTemplate {
        language: around.language,
        head: PageHead {
            title: around.title,
            root: &around.root,
            accents: &around.frame.accents,
        }
        .render(),
        frame: around.frame.render(),
        tree: TreeLandmark(around.frame).render(),
        breadcrumb: around.breadcrumb.render(),
        search: SearchBox {
            root: &around.root,
            words: &around.frame.search,
        }
        .render(),
        main,
    })
}
