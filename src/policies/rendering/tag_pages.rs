//! Policy: the tag index and each tag's page, their content rendered from their templates inside the page's layout, in the language of the catalogue.

use askama::Template;

use super::node_rendering::{Render, RenderFile, rendered};
use super::page_layout::{PageAround, whole_page};
use crate::entities::{SiteAddress, TagIndex, TagPage};
use crate::policies::{tag_index_address, tag_pages_language};

#[derive(Template)]
#[template(path = "tag_index.html")]
struct TagIndexTemplate<'a> {
    index: &'a TagIndex,
}

#[derive(Template)]
#[template(path = "tag_page.html")]
struct TagPageTemplate<'a> {
    page: &'a TagPage,
}

impl Render for TagIndex {
    fn render(&self) -> String {
        whole_page(
            &PageAround {
                language: &tag_pages_language(),
                title: &self.title,
                root: self.address().root(),
                frame: &self.frame,
                breadcrumb: &self.breadcrumb,
            },
            rendered(&TagIndexTemplate { index: self }),
        )
    }
}

impl RenderFile for TagIndex {
    fn address(&self) -> SiteAddress {
        tag_index_address()
    }
}

impl Render for TagPage {
    fn render(&self) -> String {
        whole_page(
            &PageAround {
                language: &tag_pages_language(),
                title: &self.title,
                root: self.address.root(),
                frame: &self.frame,
                breadcrumb: &self.breadcrumb,
            },
            rendered(&TagPageTemplate { page: self }),
        )
    }
}

impl RenderFile for TagPage {
    fn address(&self) -> SiteAddress {
        self.address.clone()
    }
}
