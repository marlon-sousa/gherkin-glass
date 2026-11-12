//! The anchor rule, checked against its fixtures.

use std::path::Path;

use crate::entities::{AnchoredScenario, ParsedFeature};
use crate::fixtures::{fixture, read_features};
use crate::policies::anchor_feature;

fn features() -> Vec<ParsedFeature> {
    read_features(&fixture("anchors"))
}

/// The anchors of the feature at `path`, each with the line of its block, in
/// file order, then those of the outlines' rows, each with its outline's line.
fn anchors(path: &str) -> Vec<(usize, String)> {
    let parsed = features()
        .into_iter()
        .find(|feature| feature.path == Path::new(path))
        .expect("the fixture has the feature");
    let anchored = anchor_feature(&parsed);
    let mut blocks = Vec::new();
    let mut outlines = Vec::new();
    if let Some(found) = &anchored.background {
        blocks.push((found.block.position.line, found.anchor.clone()));
    }
    add_scenarios(&anchored.scenarios, &mut blocks, &mut outlines);
    for rule in &anchored.rules {
        blocks.push((rule.rule.position.line, rule.anchor.clone()));
        if let Some(found) = &rule.background {
            blocks.push((found.block.position.line, found.anchor.clone()));
        }
        add_scenarios(&rule.scenarios, &mut blocks, &mut outlines);
    }
    for outline in outlines {
        blocks.extend(
            outline
                .rows
                .iter()
                .map(|anchor| (outline.scenario.position.line, anchor.clone())),
        );
    }
    blocks
}

fn add_scenarios<'a>(
    scenarios: &'a [AnchoredScenario<'a>],
    blocks: &mut Vec<(usize, String)>,
    outlines: &mut Vec<&'a AnchoredScenario<'a>>,
) {
    for scenario in scenarios {
        blocks.push((scenario.scenario.position.line, scenario.anchor.clone()));
        for examples in &scenario.examples {
            blocks.push((examples.block.position.line, examples.anchor.clone()));
        }
        if !scenario.examples.is_empty() {
            outlines.push(scenario);
        }
    }
}

fn expected(anchors: &[(usize, &str)]) -> Vec<(usize, String)> {
    anchors
        .iter()
        .map(|(line, anchor)| (*line, anchor.to_string()))
        .collect()
}

#[test]
fn every_block_and_outline_row_has_an_anchor_made_by_the_rule() {
    assert_eq!(
        anchors("anchors.feature"),
        expected(&[
            // Rule 31: an unnamed block takes its keyword.
            (3, "background"),
            // Rules 27 and 29: lower-cased, every gap one hyphen, none at the ends.
            (6, "withdraw-cash-quickly"),
            // Rule 28: both apostrophes are removed; rule 32: the repeat takes -2.
            (9, "the-customers-card"),
            (12, "the-customers-card-2"),
            // Rule 31: a name that leaves nothing takes its keyword.
            (15, "scenario"),
            (18, "step-2-pay"),
            (21, "withdraw-cash"),
            (24, "examples"),
            (29, "large-amounts"),
            (33, "withdraw-cash-row-2"),
            (36, "rule"),
            // Rule 32: an unnamed rule's background, after the feature's.
            (37, "background-2"),
            (40, "withdraw-cash-2"),
            // Rule 30: letters outside a to z are kept.
            (43, "étape-spéciale"),
            (44, "scenario-2"),
            // Rule 34: the feature's own name is not taken.
            (47, "anchors"),
            // Rule 27: written with a combining accent, made in NFC.
            (50, "\u{e9}tape-finale"),
            // Rule 29: marks and joiners stay inside words, and a variation
            // selector after an emoji is a gap.
            (53, "हिन्दी-में"),
            (56, "love"),
            (59, "می\u{200c}خواهم"),
            // Rule 33: examples with only a header, or no table, have no rows.
            (62, "headers-only"),
            (65, "examples-2"),
            (68, "no-table"),
            // Rule 33: rows count across examples blocks, after every other
            // anchor, and the second repeats the scenario on line 33.
            (21, "withdraw-cash-row-1"),
            (21, "withdraw-cash-row-2-2"),
            (21, "withdraw-cash-row-3"),
        ])
    );
}

#[test]
fn a_feature_in_portuguese_keeps_its_letters_and_its_keywords() {
    // Rules 30 and 31.
    assert_eq!(
        anchors("em-portugues.feature"),
        expected(&[
            (4, "contexto"),
            (7, "saque-em-conta-poupança"),
            (10, "esquema-do-cenário"),
            (13, "exemplos"),
            (10, "esquema-do-cenário-row-1"),
        ])
    );
}

#[test]
fn a_keyword_that_leaves_nothing_gives_the_block_s_kind_in_english() {
    // Rule 31: the emoji language's keywords are not letters.
    assert_eq!(
        anchors("emoji.feature"),
        expected(&[
            (4, "background"),
            (7, "scenario"),
            (10, "scenario-2"),
            (13, "scenario-outline"),
            (16, "examples"),
            (13, "scenario-outline-row-1"),
        ])
    );
}
