//! Entity: a feature as the parser returned it, before its doc strings are corrected, with its source.

use gherkin::Feature;

#[derive(Debug, Clone)]
pub struct RawFeature {
    pub feature: Feature,
    /// The text the feature was parsed from.
    pub source: String,
}
