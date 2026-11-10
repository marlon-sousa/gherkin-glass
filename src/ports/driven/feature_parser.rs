//! Port: parsing one feature file, which reads it from the input.
//!
//! The file is read by the parser rather than through the input folder port
//! because the `gherkin` crate gives its error messages only when it reads
//! the file itself.

use std::path::Path;

use gherkin::Feature;

use crate::entities::ParseFailure;

pub trait FeatureParser: Send + Sync {
    fn parse(&self, path: &Path) -> Result<Feature, ParseFailure>;
}
