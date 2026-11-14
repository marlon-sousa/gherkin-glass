use rstest::rstest;

use super::*;
use crate::entities::{PageOptions, SiteTree, text_of};
use crate::test_support::{feature_pages, site_of};

const FEATURE: &str = "\
Feature: Refunds
  Background:
    Given a customer

  Scenario: Outside
    Given x

  Rule: R
    Background: In the rule
      Given y

    Scenario Outline: Inside
      Given <a>

      Examples: First
        | a |
        | 1 |

      Examples: Second
        | a |
        | 2 |
";

fn site(text: &str) -> SiteTree {
    site_of(
        &[("billing/archive/refunds.feature", text)],
        &PageOptions::default(),
    )
}

/// The anchors of the backgrounds the scenario `node` starts with, and the first step of each.
fn from(node: SiteTreeNode<'_>) -> Vec<(String, String)> {
    node.backgrounds()
        .into_iter()
        .map(|(background, background_node)| {
            let first = background_node.steps().next().map(StepView::text);
            (background.heading.anchor.clone(), first.unwrap_or_default())
        })
        .collect()
}

#[test]
fn a_scenario_starts_with_its_feature_s_background_then_its_rule_s() {
    let site = site(FEATURE);
    let [page] = feature_pages(&site)[..] else {
        panic!("one page");
    };
    let [outside, inside] = page.scenarios().collect::<Vec<_>>()[..] else {
        panic!("two scenarios");
    };
    let feature = || ("background".to_string(), "a customer".to_string());
    assert_eq!(from(outside), [feature()]);
    assert_eq!(
        from(inside),
        [feature(), ("in-the-rule".to_string(), "y".to_string())]
    );
}

#[test]
fn a_background_without_steps_is_left_out_and_a_background_starts_with_none() {
    let site = site("Feature: F\n  Background:\n    Only words.\n\n  Scenario: S\n    Given x\n");
    let [page] = feature_pages(&site)[..] else {
        panic!("one page");
    };
    let [background, scenario] = page.blocks().collect::<Vec<_>>()[..] else {
        panic!("a background and a scenario");
    };
    assert!(scenario.backgrounds().is_empty());
    assert!(background.backgrounds().is_empty());
}

#[test]
fn a_scenario_is_in_its_rule_then_its_feature_page_and_in_its_folders() {
    let site = site(FEATURE);
    let [page] = feature_pages(&site)[..] else {
        panic!("one page");
    };
    let Some(inside) = page.scenarios().last() else {
        panic!("a scenario");
    };
    let kinds: Vec<_> = inside
        .enclosing_blocks()
        .map(|block| matches!(block.value(), SiteNode::Rule(_)))
        .collect();
    assert_eq!(kinds, [true, false]);
    assert_eq!(inside.folders(), ["billing", "archive"]);
    assert!(inside.feature_page().is_some());
    assert!(inside.folder_page().is_none());
}

#[test]
fn an_outline_holds_its_examples_and_each_block_its_steps() {
    let site = site(FEATURE);
    let [page] = feature_pages(&site)[..] else {
        panic!("one page");
    };
    let Some(inside) = page.scenarios().last() else {
        panic!("a scenario");
    };
    let names: Vec<_> = inside
        .examples()
        .map(|examples| text_of(&examples.label))
        .collect();
    assert_eq!(names, ["Examples: First", "Examples: Second"]);
    let steps: Vec<_> = inside.steps().map(StepView::text).collect();
    assert_eq!(steps, ["<a>"]);
    assert!(
        inside
            .parent()
            .is_some_and(|rule| matches!(rule.value(), SiteNode::Rule(_)))
    );
}

/// Whether the page of `before` has the same subtree as the page of `after`, each converted
/// into a site tree of its own.
fn same_page(before: &str, after: &str) -> bool {
    let (before, after) = (site(before), site(after));
    let ([before], [after]) = (&feature_pages(&before)[..], &feature_pages(&after)[..]) else {
        panic!("one page each");
    };
    before.node().same_subtree_as(after.node())
}

#[rstest]
#[case::the_same_text(FEATURE, FEATURE, true)]
#[case::a_step_changed(
    "Feature: F\n  Scenario: A\n    Given x\n",
    "Feature: F\n  Scenario: A\n    Given y\n",
    false
)]
#[case::a_scenario_added(
    "Feature: F\n  Scenario: A\n    Given x\n",
    "Feature: F\n  Scenario: A\n    Given x\n\n  Scenario: B\n",
    false
)]
#[case::a_step_moved_to_another_scenario(
    "Feature: F\n  Scenario: A\n    Given x\n\n  Scenario: B\n",
    "Feature: F\n  Scenario: A\n\n  Scenario: B\n    Given x\n",
    false
)]
fn a_page_has_the_same_subtree_as_another_tree_s_only_with_the_same_nodes_in_the_same_shape(
    #[case] before: &str,
    #[case] after: &str,
    #[case] same: bool,
) {
    assert_eq!(same_page(before, after), same);
}
