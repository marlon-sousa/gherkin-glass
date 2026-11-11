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
fn a_name_is_lower_cased_and_its_gaps_become_one_hyphen() {
    assert_eq!(
        anchor_of("Withdraw  Cash, quickly!"),
        "withdraw-cash-quickly"
    );
    assert_eq!(anchor_of("  -- The end --  "), "the-end");
    assert_eq!(anchor_of("Step 2: pay"), "step-2-pay");
}

#[test]
fn apostrophes_are_removed_not_turned_into_hyphens() {
    assert_eq!(anchor_of("The customer's card"), "the-customers-card");
    assert_eq!(anchor_of("The customer’s card"), "the-customers-card");
}

#[test]
fn letters_outside_a_to_z_are_kept_in_nfc() {
    assert_eq!(
        anchor_of("Saque em conta poupança"),
        "saque-em-conta-poupança"
    );
    // "É" written as E and a combining acute accent.
    assert_eq!(anchor_of("E\u{301}TAPE"), "\u{e9}tape");
    assert_eq!(anchor_of("Straße"), "straße");
    assert_eq!(anchor_of("हिन्दी में"), "हिन्दी-में");
    assert_eq!(anchor_of("می\u{200c}خواهم"), "می\u{200c}خواهم");
}

#[test]
fn a_mark_or_joiner_with_no_letter_before_it_is_a_gap() {
    // An emoji followed by the variation selector U+FE0F, as phones type it.
    assert_eq!(anchor_of("\u{2764}\u{fe0f} love"), "love");
    assert_eq!(anchor_of("\u{2764}\u{fe0f}"), "");
    assert_eq!(anchor_of("a \u{301}b"), "a-b");
    assert_eq!(anchor_of("a\u{200d} b"), "a-b");
    assert_eq!(anchor_of("\u{200c}a"), "a");
}

#[test]
fn a_name_of_nothing_but_gaps_leaves_nothing() {
    assert_eq!(anchor_of("???"), "");
    assert_eq!(anchor_of("📕"), "");
}

#[test]
fn repeats_take_the_first_free_number() {
    let mut taken = HashSet::new();
    let made: Vec<_> = ["a", "a", "a-2", "a"]
        .into_iter()
        .map(|anchor| unique(anchor.to_string(), &mut taken))
        .collect();
    assert_eq!(made, ["a", "a-2", "a-2-2", "a-3"]);
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
