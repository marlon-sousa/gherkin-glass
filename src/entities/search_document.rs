//! Entity: one entry of the documents search reads, a feature, a rule or a scenario.

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchDocument {
    pub kind: SearchKind,
    /// Empty for a block with no name.
    pub name: String,
    /// The name of the feature whose page the entry is on; `None` for the feature itself.
    pub feature: Option<String>,
    /// The name of the rule the scenario is in; `None` for a scenario outside a rule, a rule and a feature.
    pub rule: Option<String>,
    pub steps: Vec<String>,
    pub tags: Vec<String>,
    pub page: String,
    /// Empty for a feature, which is its page.
    pub anchor: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SearchKind {
    Feature,
    Rule,
    Scenario,
    ScenarioOutline,
}
