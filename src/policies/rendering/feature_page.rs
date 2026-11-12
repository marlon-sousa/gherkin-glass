//! Policy: a feature's page, its content rendered from its template inside the page's layout.

use askama::Template;

use super::node_rendering::{Render, RenderFile, rendered};
use super::page_layout::{PageAround, whole_page};
use crate::entities::{Page, SiteAddress};

#[derive(Template)]
#[template(path = "feature_page.html")]
struct FeatureTemplate<'a> {
    page: &'a Page,
}

impl Render for Page {
    fn render(&self) -> String {
        whole_page(
            &PageAround {
                language: &self.language,
                title: &self.title,
                root: self.address.root(),
                frame: &self.frame,
                breadcrumb: &self.breadcrumb,
            },
            rendered(&FeatureTemplate { page: self }),
        )
    }
}

impl RenderFile for Page {
    fn address(&self) -> SiteAddress {
        self.address.clone()
    }
}
