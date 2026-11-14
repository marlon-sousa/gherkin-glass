//! Policy: the page of search results, rendered from its template inside the page's layout, which leaves out the search box every other page has, since the page has its own form.

use askama::Template;

use super::breadcrumb::Place;
use super::node_rendering::{NodeOf, RenderFile, rendered};
use super::page_layout::{PageAround, SitePieces, page_with_own_search};
use crate::entities::{SearchPage, SiteAddress};
use crate::errors::FrontPageError;
use crate::policies::SEARCH_PAGE;

#[derive(Template)]
#[template(path = "search_page.html")]
struct SearchPageTemplate<'a> {
    page: &'a SearchPage,
    /// Where the form sends the words, relative to the page itself.
    address: &'a str,
}

impl RenderFile for NodeOf<'_, SearchPage> {
    fn address(&self) -> SiteAddress {
        SiteAddress::at_root(SEARCH_PAGE)
    }

    fn render_file(&self, site: &SitePieces<'_>) -> Result<String, FrontPageError> {
        let page = self.value;
        Ok(page_with_own_search(
            site,
            &PageAround {
                language: &page.language,
                title: &page.title,
                place: Place::of(self.node, &page.trail),
            },
            rendered(&SearchPageTemplate {
                page,
                address: SEARCH_PAGE,
            }),
        ))
    }
}
