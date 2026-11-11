//! Policy: the anchor rule, the anchors of a feature's page as docs/BEHAVIOUR.md, "Anchors", says.

use std::collections::HashSet;

use gherkin::{Background, Feature, Scenario};
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

use crate::entities::Anchors;
use crate::policies::example_rows;

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

pub(crate) fn make_anchors(feature: &Feature) -> Anchors {
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
        for row in 1..=example_rows(outline).len() {
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
mod tests;
