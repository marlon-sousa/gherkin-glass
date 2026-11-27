//! Policy: the home page a shell of a site without versions or of a preview holds in its `main`, the front page around ghg's part, its folders and features each rendered from their templates.

use askama::Template;

use super::contents::Contents;
use super::front_page::{PageMain, front_page};
use super::heading_parts::RenderIn;
use super::node_rendering::{Render, rendered};
use crate::entities::{FeatureEntry, FolderLink, FrontPageTemplate, HeadingLevel, HomePage};
use crate::errors::FrontPageError;

/// What the home page holds under its `h1`.
#[derive(Template)]
#[template(path = "home_page.html")]
struct HomeTemplate<'a> {
    folder: &'a HomePage,
    /// The level of its headings.
    level: HeadingLevel,
    /// The links to its folders' pages, each already rendered.
    folders: Vec<String>,
    /// Its features' entries, each already rendered.
    features: Vec<String>,
}

#[derive(Template)]
#[template(path = "home_folder.html")]
struct SubfolderLinkTemplate<'a> {
    link: &'a FolderLink,
}

#[derive(Template)]
#[template(path = "home_feature.html")]
struct FeatureEntryTemplate<'a> {
    feature: &'a FeatureEntry,
    contents: String,
}

/// `home`'s `main` and its title, written from `template`, or from ghg's own without one.
pub(super) fn home_main(
    home: &HomePage,
    template: Option<&FrontPageTemplate>,
) -> Result<PageMain, FrontPageError> {
    front_page(template, &home.title, |level| {
        rendered(&HomeTemplate {
            folder: home,
            level,
            folders: home.folders.iter().map(Render::render).collect(),
            features: home.features.iter().map(Render::render).collect(),
        })
    })
}

/// A link to a folder's page, as the home page lists it.
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
                language: Some(&self.language),
            }
            .render(),
        })
    }
}
