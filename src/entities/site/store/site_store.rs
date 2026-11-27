//! Entity: what a build stores of its site: each feature file with its object, and the results a descriptor holds that ghg works out from the build's settings, its tags and its step sentences.

use std::sync::Arc;

use crate::entities::{FeatureObject, FeaturePath, OutsideLink};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SiteStore {
    /// In the order of the feature files' paths, folder by folder.
    pub(crate) features: Vec<DescriptorFeature>,
    /// Every tag written in the input, in the order of the tag index.
    pub(crate) tags: Vec<DescriptorTag>,
    pub(crate) sentences: DescriptorSentences,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DescriptorFeature {
    pub(crate) path: FeaturePath,
    /// Its page's name, such as `billing/refunds`.
    pub(crate) page: String,
    pub(crate) object: Arc<FeatureObject>,
    /// The line of each block's keyword: the feature's, then every block inside it, depth
    /// first in file order, each block followed by its blocks, then its examples.
    pub(crate) lines: Vec<usize>,
    /// The link to its source; `None` without `source_link`.
    pub(crate) source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DescriptorTag {
    /// With its `@`.
    pub(crate) tag: String,
    /// Its page's name, such as `tags/needs-db`.
    pub(crate) page: String,
    /// `None` when no pattern of `tag_links` gives it one.
    pub(crate) link: Option<OutsideLink>,
}

/// Each kind's sentences once, sorted by their text lower-cased, then by their character codes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DescriptorSentences {
    pub(crate) given: Vec<String>,
    pub(crate) when: Vec<String>,
    pub(crate) then: Vec<String>,
}
