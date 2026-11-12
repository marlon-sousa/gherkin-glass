//! Policy: the page of step sentences, rendered from its template.

use askama::Template;

use super::node_rendering::{Render, RenderFile, page};
use super::theme_stylesheet::theme_stylesheet;
use crate::entities::{Breadcrumb, PageFrame, SentencesPage, SiteAddress};
use crate::policies::SENTENCES_PAGE;

/// At the site's root, so the shared assets are beside it.
#[derive(Template)]
#[template(path = "sentences_page.html")]
struct SentencesTemplate<'a> {
    language: &'a str,
    title: &'a str,
    root: &'a str,
    frame: &'a PageFrame,
    /// The stylesheet of the themes' colours, written into the page's head as it is.
    theme_style: String,
    breadcrumb: &'a Breadcrumb,
    page: &'a SentencesPage,
}

impl Render for SentencesPage {
    fn render(&self) -> String {
        page(&SentencesTemplate {
            language: &self.language,
            title: &self.title,
            root: "",
            frame: &self.frame,
            theme_style: theme_stylesheet(&self.frame.accents),
            breadcrumb: &self.breadcrumb,
            page: self,
        })
    }
}

impl RenderFile for SentencesPage {
    fn address(&self) -> SiteAddress {
        SiteAddress::at_root(SENTENCES_PAGE)
    }
}
