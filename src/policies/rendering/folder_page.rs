//! Policy: a folder's page, and each feature it lists, rendered from their templates inside the page's layout.

use askama::Template;

use super::contents::Contents;
use super::node_rendering::{Render, RenderFile, rendered};
use super::page_layout::{PageAround, whole_page};
use crate::entities::{FeatureEntry, FolderPage, SiteAddress};

#[derive(Template)]
#[template(path = "folder_page.html")]
struct FolderTemplate<'a> {
    folder: &'a FolderPage,
}

#[derive(Template)]
#[template(path = "folder_feature.html")]
struct FeatureEntryTemplate<'a> {
    feature: &'a FeatureEntry,
    contents: String,
}

impl Render for FolderPage {
    fn render(&self) -> String {
        whole_page(
            &PageAround {
                language: &self.language,
                title: &self.title,
                root: self.address.root(),
                frame: &self.frame,
                breadcrumb: &self.breadcrumb,
            },
            rendered(&FolderTemplate { folder: self }),
        )
    }
}

impl RenderFile for FolderPage {
    fn address(&self) -> SiteAddress {
        self.address.clone()
    }
}

impl Render for FeatureEntry {
    fn render(&self) -> String {
        rendered(&FeatureEntryTemplate {
            feature: self,
            contents: Contents {
                entries: &self.contents,
                page: self.address.to_string(),
            }
            .render(),
        })
    }
}
