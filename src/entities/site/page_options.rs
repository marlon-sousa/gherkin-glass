//! Entity: the settings that shape every page of a site.

use crate::entities::{Accents, TagLink};

/// The defaults are the configuration's: the catalogue's title, no source
/// link, every block expanded, the folder tree shown, backgrounds shown
/// inside each scenario, no tag linked to another site, and the default accents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PageOptions {
    /// The site's title; `None` for the catalogue's default.
    pub(crate) title: Option<String>,
    /// The `source_link` template, with its `{path}` and `{line}` placeholders.
    pub(crate) source_link: Option<String>,
    pub(crate) collapse_rules: bool,
    /// Scenarios, scenario outlines and backgrounds.
    pub(crate) collapse_scenarios: bool,
    /// Whether the folder tree is shown when a reader first opens the site.
    pub(crate) show_tree: bool,
    /// Whether the page is written with its backgrounds' checkbox checked, so
    /// that the backgrounds inside each scenario are expanded, or unchecked.
    pub(crate) inline_backgrounds: bool,
    /// In the order of the configuration, the first that matches a tag winning.
    pub(crate) tag_links: Vec<TagLink>,
    pub(crate) accents: Accents,
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
            accents: Accents::DEFAULT,
        }
    }
}
