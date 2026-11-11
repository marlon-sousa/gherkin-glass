use std::collections::BTreeMap;
use std::path::PathBuf;

use gherkin::{Feature, GherkinEnv};

use super::*;
use crate::entities::{
    LinkAddress, OutsideLink, ParsedFeature, StepPart, TableView, TagLink, TagView,
};
use crate::policies::{anchor_feature, name_tags, tags_of, whole_tag_pattern};

fn page(text: &str, options: &PageOptions) -> Page {
    let feature = Feature::parse(text, GherkinEnv::default()).expect("parses");
    let parsed = ParsedFeature::new(
        PathBuf::from_iter(["billing", "saque em conta.feature"]),
        feature,
        "en".to_string(),
        BTreeMap::new(),
    );
    let tags = name_tags(&tags_of(std::slice::from_ref(&parsed)));
    build_page(&anchor_feature(&parsed), options, &tags)
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
        tag_links: vec![TagLink {
            source: "@in-(.+)".into(),
            pattern: whole_tag_pattern("@in-(.+)").expect("valid"),
            address: "https://x/$1".into(),
            text: "In $1".into(),
        }],
        ..PageOptions::default()
    };
    let page = page(FEATURE, &options);
    assert_eq!(page.tags[0].outside, None);
    let rule = &page.blocks[2];
    let views: Vec<_> = [&page.tags, &page.blocks[1].tags, &rule.tags]
        .into_iter()
        .flatten()
        .collect();
    assert_eq!(views.len(), 1);
    let examples = &rule.blocks[1].blocks[0];
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
