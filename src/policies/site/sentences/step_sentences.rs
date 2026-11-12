//! Policy: the suite's step sentences, grouped and sorted, with where each is used.

use std::collections::BTreeMap;

use gherkin::{Background, Scenario, Step, StepType};

use super::sentence_slots::sentence_of;
use crate::entities::{
    Anchored, AnchoredFeature, AnchoredScenario, Sentence, SentenceGroup, SentenceUse, StepKind,
    StepSentences,
};
use crate::policies::site::heading;

pub(crate) fn find_step_sentences<'a>(
    features: impl IntoIterator<Item = &'a AnchoredFeature<'a>>,
) -> StepSentences {
    let mut features: Vec<_> = features.into_iter().collect();
    features.sort_by(|a, b| a.parsed.path.cmp(&b.parsed.path));

    let mut found: BTreeMap<SentenceKey, Vec<SentenceUse>> = BTreeMap::new();
    for anchored in features {
        let parsed = anchored.parsed;
        for FoundStep {
            anchor,
            heading,
            step,
            columns,
        } in steps_of(anchored)
        {
            let (text, values) = sentence_of(&step.value, &columns);
            found
                .entry(SentenceKey {
                    kind: kind_of(step.ty),
                    lower: text.to_lowercase(),
                    text,
                })
                .or_default()
                .push(SentenceUse {
                    path: parsed.path.clone(),
                    anchor: anchor.to_string(),
                    heading,
                    line: step.position.line,
                    values,
                });
        }
    }

    let mut groups: Vec<SentenceGroup> = Vec::new();
    for (SentenceKey { kind, text, .. }, uses) in found {
        let sentence = Sentence { text, uses };
        match groups.last_mut() {
            Some(group) if group.kind == kind => group.sentences.push(sentence),
            _ => groups.push(SentenceGroup {
                kind,
                sentences: vec![sentence],
            }),
        }
    }
    StepSentences { groups }
}

/// A sentence's place in the page: its fields, in order, sort the sentences by kind, by their
/// text lower-cased, then by the text itself.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SentenceKey {
    kind: StepKind,
    lower: String,
    text: String,
}

/// A step, with the anchor and the heading of its block, and the columns of its outline, if
/// it is in one.
struct FoundStep<'a> {
    anchor: &'a str,
    heading: String,
    step: &'a Step,
    columns: Vec<&'a str>,
}

/// What the steps of one block share.
struct StepBlock<'a> {
    anchor: &'a str,
    heading: String,
    columns: Vec<&'a str>,
}

impl<'a> StepBlock<'a> {
    fn steps(&self, steps: &'a [Step]) -> Vec<FoundStep<'a>> {
        steps.iter().map(|step| self.step(step)).collect()
    }

    fn step(&self, step: &'a Step) -> FoundStep<'a> {
        FoundStep {
            anchor: self.anchor,
            heading: self.heading.clone(),
            step,
            columns: self.columns.clone(),
        }
    }
}

/// Every step of the feature in file order.
fn steps_of<'a>(anchored: &'a AnchoredFeature<'a>) -> Vec<FoundStep<'a>> {
    let mut steps: Vec<_> = anchored
        .backgrounds()
        .flat_map(background_steps)
        .chain(anchored.all_scenarios().flat_map(scenario_steps))
        .collect();
    steps.sort_by_key(|found| found.step.span.start);
    steps
}

fn background_steps<'a>(background: &'a Anchored<'a, Background>) -> Vec<FoundStep<'a>> {
    StepBlock {
        anchor: &background.anchor,
        heading: heading(&background.block.keyword, &background.block.name),
        columns: Vec::new(),
    }
    .steps(&background.block.steps)
}

fn scenario_steps<'a>(anchored: &'a AnchoredScenario<'a>) -> Vec<FoundStep<'a>> {
    let scenario = anchored.scenario;
    StepBlock {
        anchor: &anchored.anchor,
        heading: heading(&scenario.keyword, &scenario.name),
        columns: columns_of(scenario),
    }
    .steps(&scenario.steps)
}

/// The names in the header rows of an outline's examples; none for a plain scenario.
fn columns_of(scenario: &Scenario) -> Vec<&str> {
    scenario
        .examples
        .iter()
        .filter_map(|examples| examples.table.as_ref())
        .filter_map(|table| table.rows.first())
        .flatten()
        .map(String::as_str)
        .collect()
}

fn kind_of(ty: StepType) -> StepKind {
    match ty {
        StepType::Given => StepKind::Given,
        StepType::When => StepKind::When,
        StepType::Then => StepKind::Then,
    }
}

#[cfg(test)]
mod tests;
