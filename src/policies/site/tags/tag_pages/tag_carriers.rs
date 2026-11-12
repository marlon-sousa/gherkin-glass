//! Policy: where a tag is written on a feature's page, and which of its scenarios carry it.

use crate::entities::{PageContent, RuleContent, ScenarioBlock, ScenarioContent, TagView};

/// How a scenario carries a tag: the first of these that has it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Through {
    Itself,
    Examples,
    Rule,
    Feature,
}

/// The scenarios of `page` that carry `tag`, in the order of the file.
pub(super) fn carriers<'a>(page: &'a PageContent, tag: &str) -> Vec<(&'a ScenarioBlock, Through)> {
    let feature = has(&page.page.tags, tag);
    page.scenarios
        .iter()
        .filter_map(|scenario| carrier(scenario, tag, feature_through(feature)))
        .chain(
            page.rules
                .iter()
                .flat_map(|rule| rule_carriers(rule, tag, feature)),
        )
        .collect()
}

/// The scenarios of `rule` that carry `tag`; `feature` is whether the feature does.
fn rule_carriers<'a>(
    rule: &'a RuleContent,
    tag: &str,
    feature: bool,
) -> Vec<(&'a ScenarioBlock, Through)> {
    let inherited = has(&rule.block.tags, tag)
        .then_some(Through::Rule)
        .or(feature_through(feature));
    rule.scenarios
        .iter()
        .filter_map(|scenario| carrier(scenario, tag, inherited))
        .collect()
}

/// `scenario`, with how it carries `tag`, or how it inherits it when it
/// has it neither itself nor in its examples.
fn carrier<'a>(
    scenario: &'a ScenarioContent,
    tag: &str,
    inherited: Option<Through>,
) -> Option<(&'a ScenarioBlock, Through)> {
    let through = if has(&scenario.block.tags, tag) {
        Some(Through::Itself)
    } else if in_examples(scenario, tag) {
        Some(Through::Examples)
    } else {
        inherited
    };
    through.map(|through| (&scenario.block, through))
}

fn feature_through(feature: bool) -> Option<Through> {
    feature.then_some(Through::Feature)
}

/// Whether `tag` is written anywhere on `page`.
pub(super) fn writes(page: &PageContent, tag: &str) -> bool {
    has(&page.page.tags, tag)
        || page
            .scenarios
            .iter()
            .any(|scenario| scenario_writes(scenario, tag))
        || page.rules.iter().any(|rule| rule_writes(rule, tag))
}

fn rule_writes(rule: &RuleContent, tag: &str) -> bool {
    has(&rule.block.tags, tag)
        || rule
            .scenarios
            .iter()
            .any(|scenario| scenario_writes(scenario, tag))
}

fn scenario_writes(scenario: &ScenarioContent, tag: &str) -> bool {
    has(&scenario.block.tags, tag) || in_examples(scenario, tag)
}

/// Whether one of the outline's examples blocks has `tag`.
fn in_examples(scenario: &ScenarioContent, tag: &str) -> bool {
    scenario
        .examples
        .iter()
        .any(|examples| has(&examples.tags, tag))
}

fn has(tags: &[TagView], tag: &str) -> bool {
    tags.iter().any(|view| view.text == tag)
}
