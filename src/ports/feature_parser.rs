//! Port: reading one feature file and parsing it.

use std::path::Path;

use crate::entities::RawFeature;
use crate::errors::ParseError;

pub(crate) trait FeatureParser: Send + Sync {
    fn parse(&self, path: &Path) -> Result<RawFeature, ParseError>;
}
