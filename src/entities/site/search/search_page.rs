//! Entity: the page of search results, as its template writes it, with the words the script fills in.

use crate::entities::Trail;

/// Every `String` is text, escaped when it is rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchPage {
    pub(crate) trail: Trail,
    /// The `lang` of the page.
    pub(crate) language: String,
    pub(crate) title: String,
    pub(crate) heading: String,
    pub(crate) words: SearchPageWords,
}

/// The words the script fills in, in which `{words}`, `{count}`, `{features}`, `{blocks}`,
/// `{name}` and `{place}` stand for what the script puts in their place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchPageWords {
    /// The heading for the words searched for.
    pub(crate) heading: String,
    /// The title for the words searched for.
    pub(crate) title: String,
    pub(crate) loading: String,
    pub(crate) failed: String,
    pub(crate) none: String,
    pub(crate) count: String,
    /// The paragraph that names the blocks a result is in.
    pub(crate) within: String,
    /// Between two blocks, tags or cells.
    pub(crate) separator: String,
    pub(crate) matching_steps: String,
    pub(crate) matching_tags: String,
    pub(crate) matching_cells: String,
    pub(crate) headings: KindHeadings,
}

/// A block's heading for each kind, with its name and without; an examples block without a name
/// is numbered by its place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KindHeadings {
    pub(crate) feature: String,
    pub(crate) unnamed_feature: String,
    pub(crate) rule: String,
    pub(crate) unnamed_rule: String,
    pub(crate) background: String,
    pub(crate) unnamed_background: String,
    pub(crate) scenario: String,
    pub(crate) unnamed_scenario: String,
    pub(crate) scenario_outline: String,
    pub(crate) unnamed_scenario_outline: String,
    pub(crate) examples: String,
    pub(crate) numbered_examples: String,
}
