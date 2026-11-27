use rstest::rstest;

use super::*;
use crate::entities::PageOptions;
use crate::policies::build_store;
use crate::test_support::parsed_at;

/// The object of the feature `source`.
fn object_of(source: &str) -> FeatureObject {
    let parsed = parsed_at("a.feature", source);
    let store = build_store(&[&parsed], &PageOptions::default());
    let feature = store.features.into_iter().next().expect("a feature");
    feature.object.as_ref().clone()
}

/// The hash of the first block of the feature `source`.
fn first_hash(source: &str) -> String {
    block_hash(object_of(source).blocks.first().expect("a block"))
}

const STEPS: &str = "\
Feature: F
  Scenario: S
    Given a
      | x | y |
";

// Rule 251: what ghg drops as it reads a file never changes a block's hash, and a step does.
#[rstest]
#[case::comments_and_blank_lines(
    "# language: en\nFeature: F\n\n  # a comment\n\n  Scenario: S\n    Given a\n      |x|  y  |\n",
    true
)]
#[case::moved_lines(
    "\n\n\nFeature: F\n\n\n  Scenario: S\n\n    Given a\n      | x | y |\n",
    true
)]
#[case::another_step("Feature: F\n  Scenario: S\n    Given b\n      | x | y |\n", false)]
fn a_scenario_s_hash_is_what_its_page_shows(#[case] other: &str, #[case] same: bool) {
    let hashes: Vec<String> = [STEPS, other].into_iter().map(first_hash).collect();
    assert_eq!(hashes.first() == hashes.get(1), same, "{hashes:?}");
}

// Rule 251: each kind's hash is the list the rule gives it.
#[rstest]
#[case::a_scenario(
    STEPS,
    r#"["scenario","Scenario","S","",[],[["Given ","a",[["x","y"]],null]]]"#
)]
#[case::a_rule_without_its_blocks(
    "Feature: F\n  @r\n  Rule: R\n    Scenario: S\n      Given a\n",
    r#"["rule","Rule","R","",["@r"],[]]"#
)]
#[case::an_outline_with_its_examples(
    "Feature: F\n  Scenario Outline: O\n    Given <a>\n\n    @e\n    Examples: E\n      | a |\n      | 1 |\n",
    r#"["scenario-outline","Scenario Outline","O","",[],[["Given ","<a>",null,null]],[["Examples","E","",["@e"],[["a"],["1"]]]]]"#
)]
fn a_block_s_hash_is_the_list_of_its_kind(#[case] source: &str, #[case] listed: &str) {
    assert_eq!(first_hash(source), sha256_hex(listed.as_bytes()));
}

// Rule 251: the feature's adds its language, and an examples block's its table.
#[test]
fn the_feature_s_and_an_examples_block_s_hash_add_their_own() {
    let object = object_of(
        "Feature: F\n  Scenario Outline: O\n    Given <a>\n\n    Examples:\n      | a |\n",
    );
    assert_eq!(
        feature_hash(&object),
        sha256_hex(br#"["feature","Feature","F","",[],[],"en"]"#)
    );
    let Some(BlockContent::Outline { examples, .. }) = object.blocks.first().map(|b| &b.content)
    else {
        panic!("an outline");
    };
    assert_eq!(
        examples_hash(examples.first().expect("examples")),
        sha256_hex(br#"["examples","Examples","","",[],[],[["a"]]]"#)
    );
}
