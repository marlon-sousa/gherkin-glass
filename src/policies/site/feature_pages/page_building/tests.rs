use std::collections::BTreeMap;
use std::path::PathBuf;

use gherkin::{Feature, GherkinEnv};

use super::*;
use crate::entities::{
    BackgroundContent, LinkAddress, OutsideLink, ParsedFeature, RuleContent, ScenarioContent,
    StepPart, TableView, TagView,
};
use crate::policies::site::name_tags;
use crate::test_support::{revised, tag_link};

/// The feature of `text`, in `billing/saque em conta.feature`.
fn parsed(text: &str) -> ParsedFeature {
    let feature = Feature::parse(text, GherkinEnv::default()).expect("parses");
    ParsedFeature::new(
        PathBuf::from_iter(["billing", "saque em conta.feature"]),
        feature,
        "en".to_string(),
        BTreeMap::new(),
    )
}

fn page(text: &str, options: &PageOptions) -> PageContent {
    let parsed = parsed(text);
    let tags = name_tags([&parsed]);
    build_page(revised(&parsed), options, &tags)
}

/// The blocks of a page of `FEATURE`: its background, its scenario and its rule.
fn blocks(page: &PageContent) -> (&BackgroundContent, &ScenarioContent, &RuleContent) {
    let (Some(background), [scenario], [rule]) = (
        &page.background,
        page.scenarios.as_slice(),
        page.rules.as_slice(),
    ) else {
        panic!("{page:?}");
    };
    (background, scenario, rule)
}

/// The rule's own background, and its outline.
fn in_rule(rule: &RuleContent) -> (&BackgroundContent, &ScenarioContent) {
    let (Some(background), [outline]) = (&rule.background, rule.scenarios.as_slice()) else {
        panic!("{rule:?}");
    };
    (background, outline)
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
        page.page.tags,
        [TagView {
            text: "@billing".into(),
            address: LinkAddress::to("../tags/billing.html"),
            outside: None,
        }]
    );
    assert_eq!(
        page.page.description,
        Some(vec![
            "First line,".to_string(),
            "second line.".to_string(),
            "Another paragraph.".to_string()
        ])
    );
}

#[test]
fn the_source_link_fills_its_template_with_the_encoded_path_and_the_line() {
    let options = PageOptions {
        source_link: Some("https://x/blob/main/features/{path}#L{line}".into()),
        ..PageOptions::default()
    };
    let source = page(FEATURE, &options).page.source.expect("a source link");
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
    assert_eq!(page(FEATURE, &PageOptions::default()).page.source, None);
}

#[test]
fn every_tag_links_its_page_and_the_first_outside_link_that_matches() {
    let options = PageOptions {
        tag_links: vec![tag_link("@in-(.+)", "https://x/$1", "In $1")],
        ..PageOptions::default()
    };
    let page = page(FEATURE, &options);
    assert_eq!(page.page.tags[0].outside, None);
    let (_, scenario, rule) = blocks(&page);
    let views: Vec<_> = [&page.page.tags, &scenario.block.tags, &rule.block.tags]
        .into_iter()
        .flatten()
        .collect();
    assert_eq!(views.len(), 1);
    let examples = &in_rule(rule).1.examples[0];
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
