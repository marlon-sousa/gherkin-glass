//! Policy: a folder's page, the front page among them, and each subfolder and feature it lists, rendered from their templates inside the page's layout, from the page's node and the nodes below it.

use askama::Template;

use super::breadcrumb::Place;
use super::contents::Contents;
use super::front_page::{PageHeading, PageMain, front_page};
use super::node_rendering::{Render, RenderFile, pieces, rendered};
use super::page_layout::{PageAround, SitePieces, whole_page};
use crate::entities::{
    FeatureEntry, FolderLink, FolderPage, HeadingLevel, SiteAddress, SiteFolderPage, SiteNode,
    SiteTreeNode,
};
use crate::errors::FrontPageError;

/// What a folder's page holds under its `h1`.
#[derive(Template)]
#[template(path = "folder_page.html")]
struct FolderTemplate<'a> {
    folder: &'a FolderPage,
    /// The level of its headings.
    level: HeadingLevel,
    /// The links to its subfolders' pages, each already rendered.
    folders: Vec<String>,
    /// Its features' entries, each already rendered.
    features: Vec<String>,
}

#[derive(Template)]
#[template(path = "subfolder_link.html")]
struct SubfolderLinkTemplate<'a> {
    link: &'a FolderLink,
}

#[derive(Template)]
#[template(path = "folder_feature.html")]
struct FeatureEntryTemplate<'a> {
    feature: &'a FeatureEntry,
    contents: String,
}

impl RenderFile for SiteFolderPage<'_> {
    fn address(&self) -> SiteAddress {
        self.page().address.clone()
    }

    fn render_file(&self, site: &SitePieces<'_>) -> Result<String, FrontPageError> {
        let folder = self.page();
        let content = |level| {
            rendered(&FolderTemplate {
                folder,
                level,
                folders: pieces(self.node().children().filter(is_subfolder_link)),
                features: pieces(self.node().children().filter(is_feature_entry)),
            })
        };
        let page = if folder.front_page {
            front_page(site.front_page(), &folder.title, content)?
        } else {
            PageMain {
                title: folder.title.clone(),
                main: rendered(&PageHeading {
                    heading: &folder.heading,
                    content: content(HeadingLevel::Two),
                }),
            }
        };
        Ok(whole_page(
            site,
            &PageAround {
                language: &folder.language,
                title: &page.title,
                place: Place::of(self.node(), &folder.trail),
            },
            page.main,
        ))
    }
}

/// A link to a subfolder's page, as its folder's page lists it.
impl Render for FolderLink {
    fn render(&self) -> String {
        rendered(&SubfolderLinkTemplate { link: self })
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

fn is_subfolder_link(node: &SiteTreeNode<'_>) -> bool {
    matches!(node.value(), SiteNode::SubfolderLink(_))
}

fn is_feature_entry(node: &SiteTreeNode<'_>) -> bool {
    matches!(node.value(), SiteNode::FeatureEntry(_))
}
