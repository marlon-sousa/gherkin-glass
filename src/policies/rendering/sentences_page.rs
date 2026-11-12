//! Policy: the page of step sentences, its content rendered from its template inside the page's layout.

use askama::Template;

use super::node_rendering::{RenderFile, rendered};
use super::page_layout::{PageAround, SitePieces, whole_page};
use crate::entities::{SentencesPage, SiteAddress};
use crate::policies::SENTENCES_PAGE;

#[derive(Template)]
#[template(path = "sentences_page.html")]
struct SentencesTemplate<'a> {
    page: &'a SentencesPage,
}

impl RenderFile for SentencesPage {
    fn address(&self) -> SiteAddress {
        SiteAddress::at_root(SENTENCES_PAGE)
    }

    fn render_file(&self, site: &SitePieces<'_>) -> String {
        whole_page(
            site,
            &PageAround {
                language: &self.language,
                title: &self.title,
                root: self.address().root(),
                breadcrumb: &self.breadcrumb,
            },
            rendered(&SentencesTemplate { page: self }),
        )
    }
}
