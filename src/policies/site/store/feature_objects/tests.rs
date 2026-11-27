use gherkin::{Feature, GherkinEnv};
use rstest::rstest;

use super::*;
use crate::entities::{ObjectStep, StepKind, StepPart};
use crate::test_support::parsed_at;

const SUITE: &str = "\
@shop
Feature: Refunds
  A refund gives the money back,   

  in the way it was paid.

  Background:
    Given a shop

  Scenario: Card
    Given a card

  Rule: Cash
    Background:
      Given a till

    @slow
    Scenario Outline: Coins of <value>
      When I pay <value> in \"coins\"
        | coin    |
        | <value> |

      Examples: Small
        | value |
        | 1     |

      @big
      Examples:
        | value |
        | 50    |
";

fn object() -> (FeatureObject, Vec<usize>) {
    feature_object(&parsed_at("refunds.feature", SUITE))
}

fn kinds(blocks: &[ObjectBlock]) -> Vec<BlockKind> {
    blocks.iter().map(ObjectBlock::kind).collect()
}

/// The suite's outline, inside its rule: its steps and its examples blocks.
fn outline() -> (Vec<ObjectStep>, Vec<ObjectExamples>) {
    let (object, _) = object();
    let Some(BlockContent::Rule { blocks }) = object.blocks.get(2).map(|rule| &rule.content) else {
        panic!("the rule");
    };
    let Some(BlockContent::Outline { steps, examples }) =
        blocks.get(1).map(|outline| outline.content.clone())
    else {
        panic!("the outline");
    };
    (steps, examples)
}

// Rule 249: a feature's blocks are its background, scenarios and rules, in file order, and a
// rule's its background and scenarios.
#[test]
fn the_feature_holds_its_blocks_in_file_order() {
    let (object, _) = object();
    assert_eq!(object.text.tags, ["@shop"]);
    assert_eq!(
        kinds(&object.blocks),
        [BlockKind::Background, BlockKind::Scenario, BlockKind::Rule]
    );
    let Some(BlockContent::Rule { blocks }) = object.blocks.get(2).map(|rule| &rule.content) else {
        panic!("the rule");
    };
    assert_eq!(
        kinds(blocks),
        [BlockKind::Background, BlockKind::ScenarioOutline]
    );
}

// Rule 252: the lines are the feature's, then every block's, depth first, each followed by
// its blocks and then its examples.
#[test]
fn the_lines_run_depth_first_each_block_before_its_examples() {
    assert_eq!(object().1, [2, 7, 10, 13, 14, 18, 23, 28]);
}

// Rule 249: the description without the spaces at its lines' ends and its blank lines.
#[test]
fn the_description_drops_trailing_spaces_and_blank_lines() {
    assert_eq!(
        object().0.text.description,
        "A refund gives the money back,\nin the way it was paid."
    );
}

// Rule 249: the examples are numbered from 1, hold their table, and run once for each row,
// whose anchors are numbered across the outline's examples blocks.
#[test]
fn an_outline_s_examples_hold_their_place_table_and_runs() {
    let (_, examples) = outline();
    let places: Vec<_> = examples.iter().map(|block| block.place).collect();
    assert_eq!(places, [1, 2]);
    let second = examples.get(1).expect("the second examples");
    assert_eq!(second.text.tags, ["@big"]);
    assert_eq!(second.text.name, "");
    assert_eq!(
        second.table,
        Some(vec![vec!["value".to_string()], vec!["50".to_string()]])
    );
    let anchors: Vec<_> = examples
        .iter()
        .flat_map(|block| &block.runs)
        .map(|run| run.anchor.as_str())
        .collect();
    assert_eq!(anchors, ["coins-of-value-row-1", "coins-of-value-row-2"]);
}

// Rule 249: a run's steps have the row's values in their text and their tables.
#[test]
fn a_run_s_steps_are_filled_with_its_row() {
    let (_, examples) = outline();
    let run = examples
        .get(1)
        .and_then(|examples| examples.runs.first())
        .expect("the run");
    let step = run.steps.first().expect("its step");
    assert_eq!(step.text, "I pay 50 in \"coins\"");
    assert_eq!(
        step.table,
        Some(vec![vec!["coin".to_string()], vec!["50".to_string()]])
    );
}

// Rule 250: a step holds its keyword with its space, its kind, its sentence and its slots.
#[test]
fn a_step_holds_what_ghg_works_out_of_it() {
    let (steps, _) = outline();
    let step = steps.first().expect("its step");
    assert_eq!(step.keyword, "When ");
    assert_eq!(step.text, "I pay <value> in \"coins\"");
    assert_eq!(step.kind, StepKind::When);
    assert_eq!(step.slots, ["<value>", "\"coins\""]);
    assert_eq!(step.doc_string, None);
    let joined: String = step.parts.iter().map(StepPart::text).collect();
    assert_eq!(joined, step.text);
}

// Rule 250: a keyword the language writes with no space after it keeps none.
#[test]
fn a_keyword_without_a_space_keeps_none() {
    let source = "# language: zh-CN\n功能: 退款\n  场景: 刷卡\n    假如有一张卡\n";
    let feature = Feature::parse(source, GherkinEnv::default()).expect("parses");
    let parsed = ParsedFeature::new(
        "a.feature".into(),
        feature,
        "zh-CN".to_string(),
        Default::default(),
    );
    let (object, _) = feature_object(&parsed);
    let Some(BlockContent::Scenario { steps }) = object.blocks.first().map(|b| &b.content) else {
        panic!("the scenario");
    };
    let step = steps.first().expect("its step");
    assert_eq!(
        (step.keyword.as_str(), step.text.as_str()),
        ("假如", "有一张卡")
    );
}

// Rule 249: `note` is set when a block's keyword differs from its kind's word.
#[rstest]
#[case::its_own_word("Scenario: S\n", false)]
#[case::another_word("Example: S\n", true)]
fn a_scenario_written_with_another_keyword_has_a_note(#[case] scenario: &str, #[case] note: bool) {
    let text = format!("Feature: F\n  {scenario}    Given x\n");
    let (object, _) = feature_object(&parsed_at("f.feature", &text));
    let block = object.blocks.first().expect("the scenario");
    assert_eq!(block.text.note, note);
}

// Rule 248: the language and the page's `lang`.
#[test]
fn an_english_file_s_language_and_lang_are_en() {
    let (object, _) = object();
    assert_eq!(
        (object.language.as_str(), object.lang.as_str()),
        ("en", "en")
    );
}
