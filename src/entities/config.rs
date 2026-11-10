//! Entity: what a configuration file sets, each value as the file writes it.

use std::path::PathBuf;

/// Each value is `None` when the file does not set it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Config {
    /// Relative to the configuration file's folder.
    pub(crate) input: Option<PathBuf>,
    /// Relative to the configuration file's folder.
    pub(crate) output: Option<PathBuf>,
    pub(crate) allow_unnamed: Option<bool>,
    pub(crate) source_link: Option<String>,
    pub(crate) collapse_rules: Option<bool>,
    pub(crate) collapse_scenarios: Option<bool>,
    pub(crate) title: Option<String>,
    pub(crate) show_tree: Option<bool>,
}
