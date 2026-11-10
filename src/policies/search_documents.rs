//! Policy: the documents search reads, one for each feature, rule and scenario, as docs/BEHAVIOUR.md, "Search", says.

use gherkin::{Background, Scenario};

use crate::entities::{Anchors, ParsedFeature, SearchDocument, SearchKind};
use crate::policies::page_address;

/// Above this size, search can be slow to start. MiniSearch 7.2.0, in Node 22
/// on a four-core server, took about 70 ms to index each million bytes.
pub(crate) const LARGE_SEARCH_DOCUMENTS: usize = 5_000_000;

/// The documents of every page, in the order of `pages`, and within a page in file order.
pub fn search_documents<'a>(
    pages: impl IntoIterator<Item = (&'a ParsedFeature, &'a Anchors)>,
) -> Vec<SearchDocument> {
    let mut documents = Vec::new();
    for (parsed, anchors) in pages {
        let feature = &parsed.feature;
        let address = page_address(&parsed.path);
        documents.push(SearchDocument {
            kind: SearchKind::Feature,
            name: feature.name.clone(),
            feature: None,
            rule: None,
            steps: background_steps(feature.background.as_ref()),
            tags: tags(&feature.tags),
            page: address.clone(),
            anchor: String::new(),
        });
        let mut blocks: Vec<(usize, SearchDocument)> = Vec::new();
        let scenario = |scenario: &Scenario, rule: Option<&str>| SearchDocument {
            kind: if scenario.examples.is_empty() {
                SearchKind::Scenario
            } else {
                SearchKind::ScenarioOutline
            },
            name: scenario.name.clone(),
            feature: Some(feature.name.clone()),
            rule: rule.map(str::to_string),
            steps: scenario
                .steps
                .iter()
                .map(|step| step.value.clone())
                .collect(),
            tags: scenario_tags(scenario),
            page: address.clone(),
            anchor: anchors.scenario(scenario).unwrap_or_default().to_string(),
        };
        for found in &feature.scenarios {
            blocks.push((found.span.start, scenario(found, None)));
        }
        for rule in &feature.rules {
            blocks.push((
                rule.span.start,
                SearchDocument {
                    kind: SearchKind::Rule,
                    name: rule.name.clone(),
                    feature: Some(feature.name.clone()),
                    rule: None,
                    steps: background_steps(rule.background.as_ref()),
                    tags: tags(&rule.tags),
                    page: address.clone(),
                    anchor: anchors.rule(rule).unwrap_or_default().to_string(),
                },
            ));
            for found in &rule.scenarios {
                blocks.push((found.span.start, scenario(found, Some(&rule.name))));
            }
        }
        blocks.sort_by_key(|(start, _)| *start);
        documents.extend(blocks.into_iter().map(|(_, document)| document));
    }
    documents
}

pub fn search_json(documents: &[SearchDocument]) -> String {
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
