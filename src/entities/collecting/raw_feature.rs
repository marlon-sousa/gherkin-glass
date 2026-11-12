//! Entity: a feature as the parser returned it, before its doc strings are corrected, with its source.

use gherkin::Feature;

#[derive(Debug, Clone)]
pub(crate) struct RawFeature {
    pub(crate) feature: Feature,
    /// The text the feature was parsed from.
    pub(crate) source: String,
}
