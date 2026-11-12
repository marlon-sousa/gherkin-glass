//! Policy: the tag index and each tag's page, rendered from their templates, in the language of the catalogue.

use askama::Template;

use super::node_rendering::{Render, RenderFile, page};
use super::theme_stylesheet::theme_stylesheet;
use crate::entities::{Breadcrumb, PageFrame, SiteAddress, TagIndex, TagPage};
use crate::policies::{tag_index_address, tag_pages_language};

#[derive(Template)]
#[template(path = "tag_index.html")]
struct TagIndexTemplate<'a> {
    language: &'a str,
    title: &'a str,
    /// From the page back to the site's root, for the shared assets.
    root: String,
    frame: &'a PageFrame,
    /// The stylesheet of the themes' colours, written into the page's head as it is.
    theme_style: String,
    breadcrumb: &'a Breadcrumb,
    index: &'a TagIndex,
}

#[derive(Template)]
#[template(path = "tag_page.html")]
struct TagPageTemplate<'a> {
    language: &'a str,
    title: &'a str,
    /// From the page back to the site's root, for the shared assets.
    root: String,
    frame: &'a PageFrame,
    /// The stylesheet of the themes' colours, written into the page's head as it is.
    theme_style: String,
    breadcrumb: &'a Breadcrumb,
    page: &'a TagPage,
}

impl Render for TagIndex {
    fn render(&self) -> String {
        page(&TagIndexTemplate {
            language: &tag_pages_language(),
            title: &self.title,
            root: tag_index_address().root(),
            frame: &self.frame,
            theme_style: theme_stylesheet(&self.frame.accents),
            breadcrumb: &self.breadcrumb,
            index: self,
        })
    }
}

impl RenderFile for TagIndex {
    fn address(&self) -> SiteAddress {
        tag_index_address()
    }
}

impl Render for TagPage {
    fn render(&self) -> String {
        page(&TagPageTemplate {
            language: &tag_pages_language(),
            title: &self.title,
            root: self.address.root(),
            frame: &self.frame,
            theme_style: theme_stylesheet(&self.frame.accents),
            breadcrumb: &self.breadcrumb,
            page: self,
        })
    }
}

impl RenderFile for TagPage {
    fn address(&self) -> SiteAddress {
        self.address.clone()
    }
}
