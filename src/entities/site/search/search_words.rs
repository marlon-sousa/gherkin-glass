//! Entity: gherkin-glass's own words of the search box, from the interface text catalogue.

/// The words the template writes and those the script fills in. In the
/// script's, `{count}`, `{name}`, `{feature}`, `{rule}`, `{outline}`, `{place}` and
/// `{words}` stand for what the script puts in their place.
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
    pub(crate) background: String,
    pub(crate) background_in_rule: String,
    pub(crate) examples: String,
    pub(crate) examples_in_rule: String,
    /// The name of an examples block that has none.
    pub(crate) numbered_examples: String,
    /// The link to the page of search results.
    pub(crate) all_results: String,
    /// The name of the list of results.
    pub(crate) results: String,
}
