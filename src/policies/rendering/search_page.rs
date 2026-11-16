//! Policy: the page of search results, rendered from its template inside the page's layout, with the search box every page has under its heading rather than before its content.

use askama::Template;

use super::breadcrumb::Place;
use super::node_rendering::{NodeOf, RenderFile, rendered};
use super::page_layout::{PageAround, SitePieces, page_with_own_search, search_box};
use crate::entities::{SearchPage, SiteAddress};
use crate::errors::FrontPageError;
use crate::policies::SEARCH_PAGE;

#[derive(Template)]
#[template(path = "search_page.html")]
struct SearchPageTemplate<'a> {
    page: &'a SearchPage,
    /// The search box, already rendered.
    search: String,
}

impl RenderFile for NodeOf<'_, SearchPage> {
    fn address(&self) -> SiteAddress {
        SiteAddress::at_root(SEARCH_PAGE)
    }

    fn render_file(&self, site: &SitePieces<'_>) -> Result<String, FrontPageError> {
        let page = self.value;
        let around = PageAround {
            language: &page.language,
            title: &page.title,
            place: Place::of(self.node, &page.trail),
        };
        let search = search_box(site, &around);
        Ok(page_with_own_search(
            site,
            &around,
            rendered(&SearchPageTemplate { page, search }),
        ))
    }
}
