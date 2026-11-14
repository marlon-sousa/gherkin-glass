use crate::entities::{PageOptions, SiteNode};
use crate::test_support::{feature_pages, site_of};

const FEATURE: &str = "\
Feature: Refunds
  Background:
    Given a customer

  Scenario: Outside
    Given x

  Rule: R
    Scenario Outline: Inside
      Given <a>

      Examples:
        | a |
        | 1 |

  Rule: Empty
";

#[test]
fn a_page_counts_and_lists_its_rules_and_scenarios_a_rule_s_inside_it() {
    let site = site_of(&[("refunds.feature", FEATURE)], &PageOptions::default());
    let [page] = feature_pages(&site)[..] else {
        panic!("one page");
    };
    assert_eq!(page.rule_count(), 2);
    assert_eq!(page.scenario_count(), 2);
    let contents: Vec<_> = page
        .contents()
        .into_iter()
        .map(|entry| (entry.anchor, entry.entries.len()))
        .collect();
    assert_eq!(
        contents,
        [
            ("outside_rules".to_string(), 1),
            ("r".to_string(), 1),
            ("empty".to_string(), 0)
        ]
    );
    let scenarios: Vec<_> = page
        .scenarios()
        .filter_map(|scenario| match scenario.value() {
            SiteNode::Scenario(block) => Some(block.heading.anchor.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(scenarios, ["outside", "inside"]);
    let blocks = page.blocks().count();
    assert_eq!(blocks, 4);
}
