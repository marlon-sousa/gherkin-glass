//! Policy: the suite's step sentences, grouped and sorted, with where each is used, as docs/BEHAVIOUR.md, "Step sentences", says.

use std::collections::BTreeMap;

use gherkin::{Background, Scenario, Step, StepType};

use crate::entities::{
    Anchored, AnchoredFeature, AnchoredScenario, Sentence, SentenceGroup, SentenceUse, StepKind,
    StepSentences,
};
use crate::policies::heading;
use crate::policies::sentence_slots::sentence_of;

pub(crate) fn find_step_sentences<'a>(
    features: impl IntoIterator<Item = &'a AnchoredFeature<'a>>,
) -> StepSentences {
    let mut features: Vec<_> = features.into_iter().collect();
    features.sort_by(|a, b| a.parsed.path.cmp(&b.parsed.path));

    // Keyed so that iterating sorts as rules 137 and 138 say: by kind, by the
    // text lower-cased, then by the text itself.
    let mut found: BTreeMap<(StepKind, String, String), Vec<SentenceUse>> = BTreeMap::new();
    for anchored in features {
        let parsed = anchored.parsed;
        for (anchor, heading, step, columns) in steps_of(anchored) {
            let (text, values) = sentence_of(&step.value, &columns);
            found
                .entry((kind_of(step.ty), text.to_lowercase(), text))
                .or_default()
                .push(SentenceUse {
                    path: parsed.path.clone(),
                    anchor: anchor.to_string(),
                    heading: heading.clone(),
                    line: step.position.line,
                    values,
                });
        }
    }

    let mut groups: Vec<SentenceGroup> = Vec::new();
    for ((kind, _, text), uses) in found {
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

/// A step, with the anchor and the heading of its block, and the columns of its outline, if
/// it is in one.
type FoundStep<'a> = (&'a str, String, &'a Step, Vec<&'a str>);

/// Every step of the feature in file order.
fn steps_of<'a>(anchored: &'a AnchoredFeature<'a>) -> Vec<FoundStep<'a>> {
    let mut steps: Vec<_> = anchored
        .backgrounds()
        .flat_map(background_steps)
        .chain(anchored.all_scenarios().flat_map(scenario_steps))
        .collect();
    steps.sort_by_key(|(_, _, step, _)| step.span.start);
    steps
}

fn background_steps<'a>(background: &'a Anchored<'a, Background>) -> Vec<FoundStep<'a>> {
    let heading = heading(&background.block.keyword, &background.block.name);
    background
        .block
        .steps
        .iter()
        .map(|step| {
            (
                background.anchor.as_str(),
                heading.clone(),
                step,
                Vec::new(),
            )
        })
        .collect()
}

fn scenario_steps<'a>(anchored: &'a AnchoredScenario<'a>) -> Vec<FoundStep<'a>> {
    let scenario = anchored.scenario;
    let columns = columns_of(scenario);
    let heading = heading(&scenario.keyword, &scenario.name);
    scenario
        .steps
        .iter()
        .map(|step| {
            (
                anchored.anchor.as_str(),
                heading.clone(),
                step,
                columns.clone(),
            )
        })
        .collect()
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
