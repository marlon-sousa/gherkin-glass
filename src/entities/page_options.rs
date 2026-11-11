//! Entity: the settings that shape every page of a site.

/// The defaults are the configuration's: no source link, and every block expanded.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PageOptions {
    /// The `source_link` template, with its `{path}` and `{line}` placeholders.
    pub source_link: Option<String>,
    pub collapse_rules: bool,
    /// Scenarios, scenario outlines and backgrounds.
    pub collapse_scenarios: bool,
}
