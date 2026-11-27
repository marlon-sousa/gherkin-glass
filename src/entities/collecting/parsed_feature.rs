//! Entity: a feature file, read, with its doc strings corrected.

use std::collections::BTreeMap;
use std::path::PathBuf;

use gherkin::{Feature, Step};

#[derive(Debug, Clone)]
pub(crate) struct ParsedFeature {
    /// Relative to the input folder.
    pub(crate) path: PathBuf,
    /// As the `gherkin` crate reads it, except that every doc string is
    /// corrected from the source.
    pub(crate) feature: Feature,
    /// As Gherkin names it, such as `en` or `pt`, given by the file's
    /// `# language:` line, and `en` without one.
    pub(crate) language: String,
    /// The line of each row of every examples block's table but its header, counted from 1,
    /// outline by outline in file order.
    pub(crate) row_lines: Vec<usize>,
    /// Keyed by the byte offset at which the doc string's step starts.
    media_types: BTreeMap<usize, String>,
}

impl ParsedFeature {
    pub(crate) fn new(
        path: PathBuf,
        feature: Feature,
        language: String,
        row_lines: Vec<usize>,
        media_types: BTreeMap<usize, String>,
    ) -> Self {
        ParsedFeature {
            path,
            feature,
            language,
            row_lines,
            media_types,
        }
    }

    /// The media type of `step`'s doc string, such as `json`, if it has one.
    pub(crate) fn media_type(&self, step: &Step) -> Option<&str> {
        self.media_types.get(&step.span.start).map(String::as_str)
    }
}
