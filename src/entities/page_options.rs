//! Entity: the settings that shape every page of a site.

use crate::entities::TagLink;

/// The defaults are the configuration's: the catalogue's title, no source
/// link, every block expanded, the folder tree shown, backgrounds shown
/// inside each scenario, and no tag linked to another site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageOptions {
    /// The site's title; `None` for the catalogue's default.
    pub title: Option<String>,
    /// The `source_link` template, with its `{path}` and `{line}` placeholders.
    pub source_link: Option<String>,
    pub collapse_rules: bool,
    /// Scenarios, scenario outlines and backgrounds.
    pub collapse_scenarios: bool,
    /// Whether the folder tree is shown when a reader first opens the site.
    pub show_tree: bool,
    /// Whether the page is written with each scenario showing its backgrounds' steps, or linking to them.
    pub inline_backgrounds: bool,
    /// In the order of the configuration, the first that matches a tag winning.
    pub tag_links: Vec<TagLink>,
}

impl Default for PageOptions {
    fn default() -> Self {
        PageOptions {
            title: None,
            source_link: None,
            collapse_rules: false,
            collapse_scenarios: false,
            show_tree: true,
            inline_backgrounds: true,
            tag_links: Vec::new(),
        }
    }
}
