//! Entity: what a configuration file sets, each value as the file writes it.

use std::path::PathBuf;

use crate::entities::{Colour, TagLink, VersionName};

/// Each value is `None` when the file does not set it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Config {
    /// Relative to the configuration file's folder.
    pub(crate) input: Option<PathBuf>,
    /// Relative to the configuration file's folder.
    pub(crate) output: Option<PathBuf>,
    pub(crate) allow_unnamed: Option<bool>,
    pub(crate) version_name: Option<VersionName>,
    pub(crate) replace_existing_version: Option<bool>,
    pub(crate) source_link: Option<String>,
    pub(crate) collapse_rules: Option<bool>,
    pub(crate) collapse_scenarios: Option<bool>,
    pub(crate) accent_light: Option<Colour>,
    pub(crate) accent_dark: Option<Colour>,
    pub(crate) title: Option<String>,
    /// The front page's template, relative to the configuration file's folder.
    pub(crate) front_page: Option<PathBuf>,
    pub(crate) show_tree: Option<bool>,
    pub(crate) inline_backgrounds: Option<bool>,
    pub(crate) tag_links: Option<Vec<TagLink>>,
}
