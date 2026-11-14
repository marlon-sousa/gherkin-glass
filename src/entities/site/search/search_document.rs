//! Entity: one entry of the documents search reads, a feature, a rule, a background, a scenario or an examples block.

use crate::entities::SiteAddress;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchDocument {
    pub(crate) kind: SearchKind,
    /// Empty for a block with no name.
    pub(crate) name: String,
    /// The name of the feature whose page the entry is on; `None` for the feature itself.
    pub(crate) feature: Option<String>,
    /// The name of the rule the block is in; `None` for a block outside a rule, a rule and a feature.
    pub(crate) rule: Option<String>,
    /// The outline an examples block is in; `None` for every other entry.
    pub(crate) outline: Option<OutlinePlace>,
    /// Empty for a feature, a rule and an examples block.
    pub(crate) steps: Vec<String>,
    pub(crate) tags: Vec<String>,
    /// An examples block's cells, row by row from its header; empty for every other entry.
    pub(crate) cells: Vec<String>,
    pub(crate) page: SiteAddress,
    /// Empty for a feature, which is its page.
    pub(crate) anchor: String,
}

/// Where an examples block is: its outline's name, empty for none, and its place among the
/// outline's examples blocks, from 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OutlinePlace {
    pub(crate) name: String,
    pub(crate) place: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SearchKind {
    Feature,
    Rule,
    Background,
    Scenario,
    ScenarioOutline,
    Examples,
}
