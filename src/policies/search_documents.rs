//! Policy: the documents search reads, one for each feature, rule and scenario, as docs/BEHAVIOUR.md, "Search", says.

use gherkin::{Background, Scenario};

use crate::entities::{
    AnchoredFeature, AnchoredRule, AnchoredScenario, SearchDocument, SearchKind,
};
use crate::policies::page_address;

/// Above this size, search can be slow to start. MiniSearch 7.2.0, in Node 22
/// on a four-core server, took about 70 ms to index each million bytes.
pub(crate) const LARGE_SEARCH_DOCUMENTS: usize = 5_000_000;

/// The documents of every page, in the order of `pages`, and within a page in file order.
pub(crate) fn search_documents<'a>(
    pages: impl IntoIterator<Item = &'a AnchoredFeature<'a>>,
) -> Vec<SearchDocument> {
    pages.into_iter().flat_map(page_documents).collect()
}

/// The documents of one page: its feature's, then each rule's and scenario's.
fn page_documents(anchored: &AnchoredFeature<'_>) -> Vec<SearchDocument> {
    let feature = &anchored.parsed.feature;
    let page = OnPage {
        feature: &feature.name,
        address: page_address(&anchored.parsed.path),
    };
    let feature_document = SearchDocument {
        kind: SearchKind::Feature,
        name: feature.name.clone(),
        feature: None,
        rule: None,
        steps: background_steps(feature.background.as_ref()),
        tags: tags(&feature.tags),
        page: page.address.clone(),
        anchor: String::new(),
    };
    let scenarios = anchored
        .scenarios
        .iter()
        .map(|scenario| page.scenario(scenario, None));
    let rules = anchored.rules.iter().flat_map(|rule| page.rule(rule));
    [feature_document]
        .into_iter()
        .chain(scenarios)
        .chain(rules)
        .collect()
}

/// What every document of a page shares.
struct OnPage<'a> {
    feature: &'a str,
    address: String,
}

impl OnPage<'_> {
    /// The rule's document, then its scenarios'.
    fn rule(&self, anchored: &AnchoredRule<'_>) -> Vec<SearchDocument> {
        let rule = anchored.rule;
        let document = SearchDocument {
            kind: SearchKind::Rule,
            name: rule.name.clone(),
            feature: Some(self.feature.to_string()),
            rule: None,
            steps: background_steps(rule.background.as_ref()),
            tags: tags(&rule.tags),
            page: self.address.clone(),
            anchor: anchored.anchor.clone(),
        };
        let scenarios = anchored
            .scenarios
            .iter()
            .map(|scenario| self.scenario(scenario, Some(&rule.name)));
        [document].into_iter().chain(scenarios).collect()
    }

    fn scenario(&self, anchored: &AnchoredScenario<'_>, rule: Option<&str>) -> SearchDocument {
        let scenario = anchored.scenario;
        SearchDocument {
            kind: if scenario.examples.is_empty() {
                SearchKind::Scenario
            } else {
                SearchKind::ScenarioOutline
            },
            name: scenario.name.clone(),
            feature: Some(self.feature.to_string()),
            rule: rule.map(str::to_string),
            steps: scenario
                .steps
                .iter()
                .map(|step| step.value.clone())
                .collect(),
            tags: scenario_tags(scenario),
            page: self.address.clone(),
            anchor: anchored.anchor.clone(),
        }
    }
}

#[expect(
    clippy::expect_used,
    reason = "documents are strings and lists of strings, which always serialise"
)]
pub(crate) fn search_json(documents: &[SearchDocument]) -> String {
    serde_json::to_string(documents).expect("documents are strings and lists of strings")
}

fn background_steps(background: Option<&Background>) -> Vec<String> {
    background
        .map(|found| found.steps.iter().map(|step| step.value.clone()).collect())
        .unwrap_or_default()
}

fn tags(tags: &[String]) -> Vec<String> {
    tags.iter().map(|tag| format!("@{tag}")).collect()
}

/// A scenario's own tags, then, for an outline, those of its examples blocks, each once.
fn scenario_tags(scenario: &Scenario) -> Vec<String> {
    let mut found = tags(&scenario.tags);
    for tag in scenario
        .examples
        .iter()
        .flat_map(|examples| tags(&examples.tags))
    {
        if !found.contains(&tag) {
            found.push(tag);
        }
    }
    found
}

#[cfg(test)]
mod tests;
