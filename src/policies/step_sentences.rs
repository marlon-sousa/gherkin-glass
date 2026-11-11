//! Policy: the suite's step sentences, grouped and sorted, with where each is used, as docs/BEHAVIOUR.md, "Step sentences", says.

use std::collections::BTreeMap;

use gherkin::{Scenario, Step, StepType};

use crate::entities::{
    Anchors, ParsedFeature, Sentence, SentenceGroup, SentenceUse, StepKind, StepSentences,
};
use crate::policies::heading;
use crate::policies::sentence_slots::sentence_of;

/// Each feature is paired with the anchors [`make_anchors`](crate::policies::make_anchors)
/// made for it; a pair that does not match panics.
pub(crate) fn find_step_sentences<'a>(
    features: impl IntoIterator<Item = (&'a ParsedFeature, &'a Anchors)>,
) -> StepSentences {
    let mut features: Vec<_> = features.into_iter().collect();
    features.sort_by(|(a, _), (b, _)| a.path.cmp(&b.path));

    // Keyed so that iterating sorts as rules 137 and 138 say: by kind, by the
    // text lower-cased, then by the text itself.
    let mut found: BTreeMap<(StepKind, String, String), Vec<SentenceUse>> = BTreeMap::new();
    for (parsed, anchors) in features {
        for (anchor, heading, step, columns) in steps_of(parsed, anchors) {
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

/// Every step of the feature in file order, with the anchor and the heading
/// of its block, and the columns of its outline, if it is in one.
fn steps_of<'a>(
    parsed: &'a ParsedFeature,
    anchors: &'a Anchors,
) -> Vec<(&'a str, String, &'a Step, Vec<&'a str>)> {
    let feature = &parsed.feature;
    let mut steps = Vec::new();
    let backgrounds = feature.background.iter().chain(
        feature
            .rules
            .iter()
            .filter_map(|rule| rule.background.as_ref()),
    );
    for background in backgrounds {
        let anchor = anchors
            .background(background)
            .expect("the anchors are the feature's own");
        let heading = heading(&background.keyword, &background.name);
        for step in &background.steps {
            steps.push((anchor, heading.clone(), step, Vec::new()));
        }
    }
    let scenarios = feature
        .scenarios
        .iter()
        .chain(feature.rules.iter().flat_map(|rule| &rule.scenarios));
    for scenario in scenarios {
        let anchor = anchors
            .scenario(scenario)
            .expect("the anchors are the feature's own");
        let columns = columns_of(scenario);
        let heading = heading(&scenario.keyword, &scenario.name);
        for step in &scenario.steps {
            steps.push((anchor, heading.clone(), step, columns.clone()));
        }
    }
    steps.sort_by_key(|(_, _, step, _)| step.span.start);
    steps
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
