//! Policy: the suite's step sentences, grouped and sorted, with where each is used.

use std::collections::BTreeMap;

use super::sentence_slots::sentence_of;
use crate::entities::{
    BlockHeading, Sentence, SentenceGroup, SentenceUse, SiteFeaturePage, SiteNode, SiteTreeNode,
    StepKind, StepSentences, StepView,
};

/// The sentences of the steps of `pages`, feature pages of the site tree with their blocks.
pub(crate) fn find_step_sentences(pages: &[SiteFeaturePage<'_>]) -> StepSentences {
    let mut pages: Vec<_> = pages.to_vec();
    pages.sort_by(|a, b| a.page().path.split('/').cmp(b.page().path.split('/')));

    let mut found: BTreeMap<SentenceKey, Vec<SentenceUse>> = BTreeMap::new();
    for feature in pages {
        let page = feature.page();
        for FoundStep {
            heading,
            step,
            columns,
        } in steps_of(&feature)
        {
            let (text, values) = sentence_of(&step.text(), &columns);
            found
                .entry(SentenceKey {
                    kind: step.kind,
                    lower: text.to_lowercase(),
                    text,
                })
                .or_default()
                .push(SentenceUse {
                    path: page.path.clone(),
                    page: page.address.clone(),
                    language: page.language.clone(),
                    anchor: heading.anchor.clone(),
                    heading: heading.text(),
                    line: step.line,
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

/// A step, with the heading of its block, and the columns of its outline, if it is in one.
struct FoundStep<'a> {
    heading: &'a BlockHeading,
    step: &'a StepView,
    columns: Vec<&'a str>,
}

/// Every step of the page's backgrounds and scenarios, in file order; the
/// scenarios an outline's rows make are left out.
fn steps_of<'a>(page: &SiteFeaturePage<'a>) -> Vec<FoundStep<'a>> {
    page.blocks()
        .flat_map(|block| match block.value() {
            SiteNode::Rule(_) => block.children().collect(),
            _ => vec![block],
        })
        .flat_map(block_steps)
        .collect()
}

/// The steps of the background or the scenario `block`; none for another node.
fn block_steps(block: SiteTreeNode<'_>) -> Vec<FoundStep<'_>> {
    let heading = match block.value() {
        SiteNode::Background(background) => &background.heading,
        SiteNode::Scenario(scenario) => &scenario.heading,
        _ => return Vec::new(),
    };
    let columns = columns_of(block);
    block
        .children()
        .filter_map(|child| match child.value() {
            SiteNode::Step(step) => Some(FoundStep {
                heading,
                step,
                columns: columns.clone(),
            }),
            _ => None,
        })
        .collect()
}

/// The names in the header rows of an outline's examples; none for a plain scenario.
fn columns_of(block: SiteTreeNode<'_>) -> Vec<&str> {
    block
        .children()
        .filter_map(|child| match child.value() {
            SiteNode::Examples(examples) => examples.table.as_ref(),
            _ => None,
        })
        .flat_map(|table| &table.header)
        .map(String::as_str)
        .collect()
}

#[cfg(test)]
mod tests;
