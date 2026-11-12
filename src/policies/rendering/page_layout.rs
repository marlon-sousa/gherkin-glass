//! Policy: a page of the site, the site nodes rendered once for the build and its own head, breadcrumb and search box around its main content.

use askama::Template;

use super::node_rendering::{Render, page};
use super::page_frame::TreeLandmark;
use super::page_head::PageHead;
use super::search_box::SearchBox;
use crate::entities::{Breadcrumb, PageFrame, SearchWords};

/// The site nodes every page of a version's site shows, rendered once for the build.
pub(super) struct SitePieces<'a> {
    search: &'a SearchWords,
    stylesheet: String,
    frame: String,
    tree: String,
}

/// What a page has of its own around its main content.
pub(super) struct PageAround<'a> {
    /// The `lang` of the page.
    pub(super) language: &'a str,
    pub(super) title: &'a str,
    /// From the page back to the site's root.
    pub(super) root: String,
    pub(super) breadcrumb: &'a Breadcrumb,
}

#[derive(Template)]
#[template(path = "page_layout.html")]
struct LayoutTemplate<'a> {
    language: &'a str,
    head: String,
    frame: &'a str,
    tree: &'a str,
    breadcrumb: String,
    search: String,
    main: String,
}

impl<'a> SitePieces<'a> {
    pub(super) fn of(frame: &'a PageFrame) -> Self {
        SitePieces {
            search: &frame.search,
            stylesheet: frame.accents.render(),
            frame: frame.render(),
            tree: TreeLandmark { frame }.render(),
        }
    }
}

/// The whole page whose content, already rendered, is `main`.
pub(super) fn whole_page(site: &SitePieces<'_>, around: &PageAround<'_>, main: String) -> String {
    page(&LayoutTemplate {
        language: around.language,
        head: PageHead {
            title: around.title,
            root: &around.root,
            stylesheet: &site.stylesheet,
        }
        .render(),
        frame: &site.frame,
        tree: &site.tree,
        breadcrumb: around.breadcrumb.render(),
        search: SearchBox {
            root: &around.root,
            words: site.search,
        }
        .render(),
        main,
    })
}
