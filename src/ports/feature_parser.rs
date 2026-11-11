//! Port: reading one feature file and parsing it.

use std::path::Path;

use crate::entities::{ParseFailure, RawFeature};

pub(crate) trait FeatureParser: Send + Sync {
    fn parse(&self, path: &Path) -> Result<RawFeature, ParseFailure>;
}
