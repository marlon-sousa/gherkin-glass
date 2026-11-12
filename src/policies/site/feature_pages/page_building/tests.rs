use std::collections::BTreeMap;
use std::path::PathBuf;

use gherkin::{Feature, GherkinEnv};

use super::*;
use crate::entities::{
    BackgroundBlock, LinkAddress, OutlineParts, OutsideLink, ParsedFeature, RuleBlock, StepPart,
    TableView, TagView,
};
use crate::policies::site::{anchor_feature, name_tags, tags_of};
use crate::test_support::tag_link;

fn page(text: &str, options: &PageOptions) -> Page {
    let feature = Feature::parse(text, GherkinEnv::default()).expect("parses");
    let parsed = ParsedFeature::new(
        PathBuf::from_iter(["billing", "saque em conta.feature"]),
        feature,
        "en".to_string(),
        BTreeMap::new(),
    );
    let tags = name_tags(&tags_of(std::slice::from_ref(&parsed)));
    build_page(
        &anchor_feature(&parsed),
        options,
        &tags,
        &["billing".to_string()],
    )
}

/// The blocks of a page of `FEATURE`: its background, its scenario and its rule.
fn blocks(page: &Page) -> (&BackgroundBlock, &ScenarioBlock, &RuleBlock) {
    let [
        FeatureBlock::Background(background),
        FeatureBlock::Scenario(scenario),
        FeatureBlock::Rule(rule),
    ] = page.blocks.as_slice()
    else {
        panic!("{:?}", page.blocks);
    };
    (background, scenario, rule)
}

/// The rule's own background, and its outline.
fn in_rule(rule: &RuleBlock) -> (&BackgroundBlock, &ScenarioBlock) {
    let (Some(background), [outline]) = (&rule.background, rule.scenarios.as_slice()) else {
        panic!("{rule:?}");
    };
    (background, outline)
}

/// What only an outline has.
fn parts(outline: &ScenarioBlock) -> &OutlineParts {
    outline.outline.as_ref().expect("an outline")
}

const FEATURE: &str = "\
@billing
Feature: Refunds
  First line,
  second line.

  Another paragraph.

  Background:
    Given a customer

  Scenario: Outside
    Given x

  Rule: R
    Background: In the rule
      Given y

    Scenario Outline: Inside
      Given <n>

      @in-rule
      Examples:
        | n |
        | 1 |
";

mod backgrounds;
mod blocks;

#[test]
fn the_description_is_a_paragraph_that_keeps_its_lines() {
    let page = page(FEATURE, &PageOptions::default());
    assert_eq!(
        page.tags,
        [TagView {
            text: "@billing".into(),
            address: LinkAddress::to("../tags/billing.html"),
            outside: None,
        }]
    );
    assert_eq!(
        page.description,
        Some(vec![
            "First line,".to_string(),
            "second line.".to_string(),
            "Another paragraph.".to_string()
        ])
    );
}

#[test]
fn the_contents_list_rules_and_scenarios_with_a_rule_s_scenarios_inside_it() {
    let page = page(FEATURE, &PageOptions::default());
    let entries: Vec<_> = page
        .contents
        .iter()
        .map(|e| (e.anchor.as_str(), e.entries.len()))
        .collect();
    assert_eq!(entries, [("outside", 0), ("r", 1)]);
    assert_eq!(
        page.contents[1].entries[0].heading,
        "Scenario Outline: Inside"
    );
}

#[test]
fn the_source_link_fills_its_template_with_the_encoded_path_and_the_line() {
    let options = PageOptions {
        source_link: Some("https://x/blob/main/features/{path}#L{line}".into()),
        ..PageOptions::default()
    };
    let source = page(FEATURE, &options).source.expect("a source link");
    assert_eq!(
        source.address,
        "https://x/blob/main/features/billing/saque%20em%20conta.feature#L2"
    );
    assert_eq!(
        (
            source.before.as_str(),
            source.path.as_str(),
            source.after.as_str()
        ),
        ("Source: ", "billing/saque em conta.feature", "")
    );
    assert_eq!(page(FEATURE, &PageOptions::default()).source, None);
}

#[test]
fn every_tag_links_its_page_and_the_first_outside_link_that_matches() {
    let options = PageOptions {
        tag_links: vec![tag_link("@in-(.+)", "https://x/$1", "In $1")],
        ..PageOptions::default()
    };
    let page = page(FEATURE, &options);
    assert_eq!(page.tags[0].outside, None);
    let (_, scenario, rule) = blocks(&page);
    let views: Vec<_> = [&page.tags, &scenario.tags, &rule.tags]
        .into_iter()
        .flatten()
        .collect();
    assert_eq!(views.len(), 1);
    let examples = &parts(in_rule(rule).1).examples[0];
    let tag = &examples.tags[0];
    assert_eq!(tag.address.to_string(), "../tags/in-rule.html");
    assert_eq!(
        tag.outside,
        Some(OutsideLink {
            address: "https://x/rule".into(),
            text: "In rule".into(),
        })
    );
}
