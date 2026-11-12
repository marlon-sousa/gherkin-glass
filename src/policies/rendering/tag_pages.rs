//! Policy: the tag index and each tag's page, their content rendered from their templates inside the page's layout, in the language their nodes hold.

use askama::Template;

use super::node_rendering::{Render, RenderFile, rendered};
use super::page_layout::{PageAround, SitePieces, whole_page};
use crate::entities::{SiteAddress, TagIndex, TagPage};

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

impl RenderFile for TagIndex {
    fn address(&self) -> SiteAddress {
        self.address.clone()
    }

    fn render_file(&self, site: &SitePieces<'_>) -> String {
        whole_page(
            site,
            &PageAround {
                language: &self.language,
                title: &self.title,
                root: self.address.root(),
                breadcrumb: &self.breadcrumb,
            },
            rendered(&TagIndexTemplate { index: self }),
        )
    }
}

impl RenderFile for TagPage {
    fn address(&self) -> SiteAddress {
        self.address.clone()
    }

    fn render_file(&self, site: &SitePieces<'_>) -> String {
        whole_page(
            site,
            &PageAround {
                language: &self.language,
                title: &self.title,
                root: self.address.root(),
                breadcrumb: &self.breadcrumb,
            },
            rendered(&TagPageTemplate { page: self }),
        )
    }
}
