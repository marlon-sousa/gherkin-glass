//! Policy: where a tag is written on a feature's page, and which of its scenarios carry it.

use crate::entities::{FeatureBlock, Page, ScenarioBlock, TagView};

/// How a scenario carries a tag: the first of these that has it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Through {
    Itself,
    Examples,
    Rule,
    Feature,
}

/// The scenarios of `page` that carry `tag`, in the order of the file.
pub(super) fn carriers<'a>(page: &'a Page, tag: &str) -> Vec<(&'a ScenarioBlock, Through)> {
    let feature = has(&page.tags, tag);
    page.blocks
        .iter()
        .flat_map(|block| block_carriers(block, tag, feature))
        .collect()
}

/// The scenarios `block` is or holds that carry `tag`; `feature` is whether the feature does.
fn block_carriers<'a>(
    block: &'a FeatureBlock,
    tag: &str,
    feature: bool,
) -> Vec<(&'a ScenarioBlock, Through)> {
    match block {
        FeatureBlock::Background(_) => Vec::new(),
        FeatureBlock::Rule(rule) => {
            let inherited = has(&rule.tags, tag)
                .then_some(Through::Rule)
                .or(feature_through(feature));
            rule.scenarios
                .iter()
                .filter_map(|scenario| carrier(scenario, tag, inherited))
                .collect()
        }
        FeatureBlock::Scenario(scenario) => carrier(scenario, tag, feature_through(feature))
            .into_iter()
            .collect(),
    }
}

/// `scenario`, with how it carries `tag`, or how it inherits it when it
/// has it neither itself nor in its examples.
fn carrier<'a>(
    scenario: &'a ScenarioBlock,
    tag: &str,
    inherited: Option<Through>,
) -> Option<(&'a ScenarioBlock, Through)> {
    let through = if has(&scenario.tags, tag) {
        Some(Through::Itself)
    } else if in_examples(scenario, tag) {
        Some(Through::Examples)
    } else {
        inherited
    };
    through.map(|through| (scenario, through))
}

fn feature_through(feature: bool) -> Option<Through> {
    feature.then_some(Through::Feature)
}

/// Whether `tag` is written anywhere on `page`.
pub(super) fn writes(page: &Page, tag: &str) -> bool {
    has(&page.tags, tag) || page.blocks.iter().any(|block| block_writes(block, tag))
}

fn block_writes(block: &FeatureBlock, tag: &str) -> bool {
    match block {
        FeatureBlock::Background(_) => false,
        FeatureBlock::Rule(rule) => {
            has(&rule.tags, tag)
                || rule
                    .scenarios
                    .iter()
                    .any(|scenario| scenario_writes(scenario, tag))
        }
        FeatureBlock::Scenario(scenario) => scenario_writes(scenario, tag),
    }
}

fn scenario_writes(scenario: &ScenarioBlock, tag: &str) -> bool {
    has(&scenario.tags, tag) || in_examples(scenario, tag)
}

/// Whether one of the outline's examples blocks has `tag`.
fn in_examples(scenario: &ScenarioBlock, tag: &str) -> bool {
    scenario
        .outline
        .iter()
        .flat_map(|outline| &outline.examples)
        .any(|examples| has(&examples.tags, tag))
}

fn has(tags: &[TagView], tag: &str) -> bool {
    tags.iter().any(|view| view.text == tag)
}
