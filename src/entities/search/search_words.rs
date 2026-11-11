//! Entity: gherkin-glass's own words of the search box, from the interface text catalogue.

/// The words the template writes and those the script fills in. In the
/// script's, `{count}`, `{name}`, `{feature}` and `{rule}` stand for what the
/// script puts in their place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchWords {
    /// The most results the list shows.
    pub(crate) shown: usize,
    pub(crate) label: String,
    pub(crate) loading: String,
    pub(crate) failed: String,
    pub(crate) none: String,
    pub(crate) count: String,
    pub(crate) count_limited: String,
    pub(crate) no_name: String,
    pub(crate) feature: String,
    pub(crate) rule: String,
    pub(crate) scenario: String,
    pub(crate) scenario_in_rule: String,
    pub(crate) scenario_outline: String,
    pub(crate) scenario_outline_in_rule: String,
}
