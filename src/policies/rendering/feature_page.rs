//! Policy: a feature's page, its content rendered from its template inside the page's layout, from the page node and the blocks below it.

use askama::Template;

use super::node_rendering::{Render, RenderFile, pieces, rendered};
use super::page_layout::{PageAround, SitePieces, whole_page};
use crate::entities::{ContentsEntry, Page, SiteAddress, SiteFeaturePage};

#[derive(Template)]
#[template(path = "feature_page.html")]
struct FeatureTemplate<'a> {
    page: &'a Page,
    contents: Vec<ContentsEntry>,
    /// Its blocks, already rendered; empty for a page with none.
    blocks: String,
}

impl RenderFile for SiteFeaturePage<'_> {
    fn address(&self) -> SiteAddress {
        self.page().address.clone()
    }

    fn render_file(&self, site: &SitePieces<'_>) -> String {
        let page = self.page();
        whole_page(
            site,
            &PageAround {
                language: &page.language,
                title: &page.title,
                root: page.address.root(),
                breadcrumb: &page.breadcrumb,
            },
            rendered(&FeatureTemplate {
                page,
                contents: self.contents(),
                blocks: pieces(self.blocks()).join("\n"),
            }),
        )
    }
}
