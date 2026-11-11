//! Policy: the anchor rule, the anchors of a feature's page as docs/BEHAVIOUR.md, "Anchors", says.

use std::collections::HashSet;

use gherkin::{Background, Feature, Scenario};
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

use crate::entities::Anchors;

/// The kinds of block that have an anchor, each named in English for when
/// neither its name nor its keyword leaves anything.
#[derive(Debug, Clone, Copy)]
enum Kind {
    Background,
    Rule,
    Scenario,
    ScenarioOutline,
    Examples,
}

/// One block, before its anchor is made unique on the page.
struct Block<'a> {
    start: usize,
    name: &'a str,
    keyword: &'a str,
    kind: Kind,
}

pub fn make_anchors(feature: &Feature) -> Anchors {
    let mut blocks = Vec::new();
    let mut outlines = Vec::new();
    if let Some(background) = &feature.background {
        blocks.push(Block::background(background));
    }
    add_scenarios(&feature.scenarios, &mut blocks, &mut outlines);
    for rule in &feature.rules {
        blocks.push(Block {
            start: rule.span.start,
            name: &rule.name,
            keyword: &rule.keyword,
            kind: Kind::Rule,
        });
        if let Some(background) = &rule.background {
            blocks.push(Block::background(background));
        }
        add_scenarios(&rule.scenarios, &mut blocks, &mut outlines);
    }
    blocks.sort_by_key(|block| block.start);
    outlines.sort_by_key(|outline| outline.span.start);

    let mut anchors = Anchors::default();
    let mut taken = HashSet::new();
    for block in &blocks {
        let anchor = unique(block.anchor(), &mut taken);
        anchors.insert_block(block.start, anchor);
    }
    for outline in outlines {
        let base = anchors.scenario(outline).unwrap_or_default().to_string();
        let rows = outline
            .examples
            .iter()
            .filter_map(|examples| examples.table.as_ref())
            .map(|table| table.rows.len().saturating_sub(1))
            .sum::<usize>();
        for row in 1..=rows {
            let anchor = unique(format!("{base}-row-{row}"), &mut taken);
            anchors.insert_row(outline.span.start, row, anchor);
        }
    }
    anchors
}

fn add_scenarios<'a>(
    scenarios: &'a [Scenario],
    blocks: &mut Vec<Block<'a>>,
    outlines: &mut Vec<&'a Scenario>,
) {
    for scenario in scenarios {
        let outline = !scenario.examples.is_empty();
        blocks.push(Block {
            start: scenario.span.start,
            name: &scenario.name,
            keyword: &scenario.keyword,
            kind: if outline {
                Kind::ScenarioOutline
            } else {
                Kind::Scenario
            },
        });
        if outline {
            outlines.push(scenario);
        }
        for examples in &scenario.examples {
            blocks.push(Block {
                start: examples.span.start,
                name: examples.name.as_deref().unwrap_or_default(),
                keyword: &examples.keyword,
                kind: Kind::Examples,
            });
        }
    }
}

impl<'a> Block<'a> {
    fn background(background: &'a Background) -> Self {
        Block {
            start: background.span.start,
            name: &background.name,
            keyword: &background.keyword,
            kind: Kind::Background,
        }
    }

    /// Rules 27 to 31.
    fn anchor(&self) -> String {
        [anchor_of(self.name), anchor_of(self.keyword)]
            .into_iter()
            .find(|anchor| !anchor.is_empty())
            .unwrap_or_else(|| self.kind.english().to_string())
    }
}

impl Kind {
    fn english(self) -> &'static str {
        match self {
            Kind::Background => "background",
            Kind::Rule => "rule",
            Kind::Scenario => "scenario",
            Kind::ScenarioOutline => "scenario-outline",
            Kind::Examples => "examples",
        }
    }
}

/// Rule 32: `anchor`, or the first of it followed by `-2`, `-3` and so on that is not taken.
pub(crate) fn unique(anchor: String, taken: &mut HashSet<String>) -> String {
    let mut candidate = anchor.clone();
    let mut number = 2;
    while taken.contains(&candidate) {
        candidate = format!("{anchor}-{number}");
        number += 1;
    }
    taken.insert(candidate.clone());
    candidate
}

/// Rules 27 to 29 on one name or keyword; empty when nothing is left.
pub(crate) fn anchor_of(text: &str) -> String {
    // Lower-casing can leave text that is no longer in NFC, so it is normalised again.
    let lower: String = text
        .nfc()
        .collect::<String>()
        .to_lowercase()
        .nfc()
        .collect();
    let characters: Vec<char> = lower
        .chars()
        .filter(|c| !matches!(c, '\'' | '\u{2019}'))
        .collect();
    let mut anchor = String::with_capacity(lower.len());
    // Whether the last character looked at was kept, so that a mark or a joiner may follow it.
    let mut joined = false;
    for (index, &character) in characters.iter().enumerate() {
        let keep = if character.is_alphanumeric() {
            if !joined && !anchor.is_empty() {
                anchor.push('-');
            }
            true
        } else if is_combining_mark(character) {
            joined
        } else if is_joiner(character) {
            joined
                && characters
                    .get(index + 1)
                    .is_some_and(|c| c.is_alphanumeric())
        } else {
            false
        };
        if keep {
            anchor.push(character);
        }
        joined = keep;
    }
    anchor
}

/// The zero width non-joiner and joiner, which Persian and the scripts of India write inside words.
fn is_joiner(character: char) -> bool {
    matches!(character, '\u{200c}' | '\u{200d}')
}

#[cfg(test)]
mod tests {
    use gherkin::GherkinEnv;

    use super::*;

    fn feature(source: &str) -> Feature {
        Feature::parse(source, GherkinEnv::default()).expect("parsed")
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
        let anchors = make_anchors(&feature);
        let outline = &feature.scenarios[0];
        assert_eq!(anchors.scenario(outline), Some("pay"));
        assert_eq!(anchors.examples(&outline.examples[0]), Some("examples"));
        assert_eq!(anchors.examples(&outline.examples[1]), Some("more"));
        assert_eq!(anchors.scenario(&feature.scenarios[1]), Some("pay-row-2"));
        assert_eq!(anchors.row(outline, 1), Some("pay-row-1"));
        assert_eq!(anchors.row(outline, 2), Some("pay-row-2-2"));
        assert_eq!(anchors.row(outline, 3), Some("pay-row-3"));
        assert_eq!(anchors.row(outline, 4), None);
    }

    #[test]
    fn an_unnamed_block_takes_its_keyword_and_then_its_kind() {
        let feature = feature("# language: em\n📚: Livro\n  📕:\n    😐 x\n  📕: ???\n    😐 y\n");
        let anchors = make_anchors(&feature);
        assert_eq!(anchors.scenario(&feature.scenarios[0]), Some("scenario"));
        assert_eq!(anchors.scenario(&feature.scenarios[1]), Some("scenario-2"));
    }
}
