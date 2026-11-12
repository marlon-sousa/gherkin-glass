//! Entity: a feature the input tree holds, with the revision of its file, which the converter compares with the one a page was converted from.

use crate::entities::{ParsedFeature, Revision};

#[derive(Debug, Clone, Copy)]
pub(crate) struct RevisedFeature<'a> {
    pub(crate) feature: &'a ParsedFeature,
    pub(crate) revision: Revision,
}
