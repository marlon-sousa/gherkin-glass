//! Policy: a folder's page, rendered from its template.

use askama::Template;

use super::node_rendering::{Render, RenderFile, page};
use super::theme_stylesheet::theme_stylesheet;
use crate::entities::{Breadcrumb, FolderPage, PageFrame, SiteAddress};

#[derive(Template)]
#[template(path = "folder_page.html")]
struct FolderTemplate<'a> {
    language: &'a str,
    title: &'a str,
    /// From the page back to the site's root, for the shared assets.
    root: String,
    frame: &'a PageFrame,
    /// The stylesheet of the themes' colours, written into the page's head as it is.
    theme_style: String,
    breadcrumb: &'a Breadcrumb,
    folder: &'a FolderPage,
}

impl Render for FolderPage {
    fn render(&self) -> String {
        page(&FolderTemplate {
            language: &self.language,
            title: &self.title,
            root: self.address.root(),
            frame: &self.frame,
            theme_style: theme_stylesheet(&self.frame.accents),
            breadcrumb: &self.breadcrumb,
            folder: self,
        })
    }
}

impl RenderFile for FolderPage {
    fn address(&self) -> SiteAddress {
        self.address.clone()
    }
}
