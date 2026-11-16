//! Policy: a page of the site, the site nodes rendered once for the build, with the front page's template, and its own head, breadcrumb and search box around its main content.

use askama::Template;

use super::breadcrumb::Place;
use super::node_rendering::{Render, page};
use super::page_frame::{PageFooter, TreeLandmark};
use super::page_head::PageHead;
use super::search_box::SearchBox;
use crate::entities::{FrontPageTemplate, PageFrame};

/// The site nodes every page of a version's site shows, rendered once for the build.
pub(super) struct SitePieces<'a> {
    page_frame: &'a PageFrame,
    front_page: Option<&'a FrontPageTemplate>,
    stylesheet: String,
    frame: String,
    tree: String,
    footer: String,
}

/// What a page has of its own around its main content.
pub(super) struct PageAround<'a> {
    /// The `lang` of the page.
    pub(super) language: &'a str,
    pub(super) title: &'a str,
    pub(super) place: Place<'a>,
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
    only_note: Option<&'a str>,
    footer: &'a str,
}

impl<'a> SitePieces<'a> {
    /// `front_page` is the front page's template, `None` for ghg's own.
    pub(super) fn of(frame: &'a PageFrame, front_page: Option<&'a FrontPageTemplate>) -> Self {
        SitePieces {
            page_frame: frame,
            front_page,
            stylesheet: frame.accents.render(),
            frame: frame.render(),
            tree: TreeLandmark { frame }.render(),
            footer: PageFooter { frame }.render(),
        }
    }
}

impl SitePieces<'_> {
    /// The frame every page shares, for a page that shows only some of its parts.
    pub(super) fn page_frame(&self) -> &PageFrame {
        self.page_frame
    }

    /// The front page's template; `None` for ghg's own.
    pub(super) fn front_page(&self) -> Option<&FrontPageTemplate> {
        self.front_page
    }

    /// The themes' stylesheet, already rendered from the site's accents.
    pub(super) fn stylesheet(&self) -> &str {
        &self.stylesheet
    }
}

/// The whole page whose content, already rendered, is `main`.
pub(super) fn whole_page(site: &SitePieces<'_>, around: &PageAround<'_>, main: String) -> String {
    let search = search_box(site, around);
    layout(site, around, &around.place.root(), search, main)
}

/// The search box of the page `around` places, rendered.
pub(super) fn search_box(site: &SitePieces<'_>, around: &PageAround<'_>) -> String {
    SearchBox {
        root: &around.place.root(),
        words: &site.page_frame.search,
    }
    .render()
}

/// The whole page whose content, already rendered, is `main`, and holds the search box itself,
/// so that the page has none before it.
pub(super) fn page_with_own_search(
    site: &SitePieces<'_>,
    around: &PageAround<'_>,
    main: String,
) -> String {
    layout(site, around, &around.place.root(), String::new(), main)
}

/// The whole page whose search box, already rendered, is `search`, empty for none, and whose
/// content is `main`; `root` leads from the page back to the site's root.
fn layout(
    site: &SitePieces<'_>,
    around: &PageAround<'_>,
    root: &str,
    search: String,
    main: String,
) -> String {
    page(&LayoutTemplate {
        language: around.language,
        head: PageHead {
            title: around.title,
            root,
            stylesheet: &site.stylesheet,
            indexing: site.page_frame.indexing,
        }
        .render(),
        frame: &site.frame,
        tree: &site.tree,
        breadcrumb: around.place.breadcrumb(&site.page_frame.breadcrumb),
        search,
        main,
        only_note: site.page_frame.only_note.as_deref(),
        footer: &site.footer,
    })
}
