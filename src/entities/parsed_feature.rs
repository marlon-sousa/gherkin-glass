//! Entity: a feature file, read, with its doc strings corrected.

use std::collections::BTreeMap;
use std::path::PathBuf;

use gherkin::{Feature, Step};

#[derive(Debug, Clone)]
pub(crate) struct ParsedFeature {
    /// Relative to the input folder.
    pub(crate) path: PathBuf,
    /// As the `gherkin` crate reads it, except that every doc string is
    /// corrected as docs/BEHAVIOUR.md, "Doc strings", says.
    pub(crate) feature: Feature,
    /// As Gherkin names it, such as `en` or `pt`, given by the file's
    /// `# language:` line, and `en` without one.
    pub(crate) language: String,
    /// Keyed by the byte offset at which the doc string's step starts.
    media_types: BTreeMap<usize, String>,
}

impl ParsedFeature {
    pub(crate) fn new(
        path: PathBuf,
        feature: Feature,
        language: String,
        media_types: BTreeMap<usize, String>,
    ) -> Self {
        ParsedFeature {
            path,
            feature,
            language,
            media_types,
        }
    }

    /// The media type of `step`'s doc string, such as `json`, if it has one.
    pub(crate) fn media_type(&self, step: &Step) -> Option<&str> {
        self.media_types.get(&step.span.start).map(String::as_str)
    }
}
