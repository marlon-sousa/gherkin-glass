//! Entity: one entry of the documents search reads, a feature, a rule or a scenario.

use crate::entities::SiteAddress;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchDocument {
    pub(crate) kind: SearchKind,
    /// Empty for a block with no name.
    pub(crate) name: String,
    /// The name of the feature whose page the entry is on; `None` for the feature itself.
    pub(crate) feature: Option<String>,
    /// The name of the rule the scenario is in; `None` for a scenario outside a rule, a rule and a feature.
    pub(crate) rule: Option<String>,
    pub(crate) steps: Vec<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) page: SiteAddress,
    /// Empty for a feature, which is its page.
    pub(crate) anchor: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SearchKind {
    Feature,
    Rule,
    Scenario,
    ScenarioOutline,
}
