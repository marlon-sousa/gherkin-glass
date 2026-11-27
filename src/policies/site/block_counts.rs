//! Policy: how many rules and scenarios a feature's object holds, and which of its blocks are rules and scenarios, as the home page and a build's report count them.

use crate::entities::{BlockContent, BlockKind, FeatureObject, ObjectBlock};

pub(super) fn is_rule(block: &ObjectBlock) -> bool {
    block.kind() == BlockKind::Rule
}

/// Whether `block` is a scenario or an outline.
pub(super) fn is_scenario(block: &ObjectBlock) -> bool {
    matches!(
        block.kind(),
        BlockKind::Scenario | BlockKind::ScenarioOutline
    )
}

pub(super) fn rule_count(object: &FeatureObject) -> usize {
    object.blocks.iter().filter(|block| is_rule(block)).count()
}

/// Its scenarios and outlines, each outline once, outside every rule and inside each.
pub(super) fn scenario_count(object: &FeatureObject) -> usize {
    object
        .blocks
        .iter()
        .map(|block| match &block.content {
            BlockContent::Rule { blocks } => {
                blocks.iter().filter(|inner| is_scenario(inner)).count()
            }
            _ => usize::from(is_scenario(block)),
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::PageOptions;
    use crate::policies::build_store;
    use crate::test_support::parsed_at;

    // Rule 82: an outline counts once, inside a rule as outside.
    #[test]
    fn scenarios_and_outlines_count_once_inside_and_outside_rules() {
        let parsed = parsed_at(
            "a.feature",
            "Feature: F\n  Scenario: A\n    Given a\n  Rule: R\n    Scenario Outline: B\n      Given <x>\n      Examples:\n        | x |\n        | 1 |\n        | 2 |\n    Scenario: C\n      Given c\n",
        );
        let store = build_store(&[&parsed], &PageOptions::default());
        let object = store
            .features
            .first()
            .map(|feature| feature.object.as_ref());
        let object = object.expect("a feature");
        assert_eq!((rule_count(object), scenario_count(object)), (1, 3));
    }
}
