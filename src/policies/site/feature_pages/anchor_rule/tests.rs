mod against_fixtures;

use std::collections::BTreeMap;
use std::path::PathBuf;

use gherkin::{Feature, GherkinEnv};

use super::*;

fn feature(source: &str) -> ParsedFeature {
    ParsedFeature::new(
        PathBuf::from("f.feature"),
        Feature::parse(source, GherkinEnv::default()).expect("parsed"),
        "en".to_string(),
        BTreeMap::new(),
    )
}

#[test]
fn outline_rows_count_across_examples_and_come_after_the_other_anchors() {
    let feature = feature(
        "Feature: F\n\
         \x20 Scenario Outline: Pay\n\
         \x20   Given <n>\n\
         \x20   Examples:\n\
         \x20     | n |\n\
         \x20     | 1 |\n\
         \x20   Examples: More\n\
         \x20     | n |\n\
         \x20     | 2 |\n\
         \x20     | 3 |\n\
         \x20 Scenario: Pay row 2\n\
         \x20   Given x\n",
    );
    let anchored = anchor_feature(&feature);
    let outline = &anchored.scenarios[0];
    assert_eq!(outline.anchor, "pay");
    assert_eq!(outline.examples[0].anchor, "examples");
    assert_eq!(outline.examples[1].anchor, "more");
    assert_eq!(anchored.scenarios[1].anchor, "pay-row-2");
    assert_eq!(outline.rows, ["pay-row-1", "pay-row-2-2", "pay-row-3"]);
}

#[test]
fn an_unnamed_block_takes_its_keyword_and_then_its_kind() {
    let feature = feature("# language: em\n📚: Livro\n  📕:\n    😐 x\n  📕: ???\n    😐 y\n");
    let anchored = anchor_feature(&feature);
    assert_eq!(anchored.scenarios[0].anchor, "scenario");
    assert_eq!(anchored.scenarios[1].anchor, "scenario-2");
}
