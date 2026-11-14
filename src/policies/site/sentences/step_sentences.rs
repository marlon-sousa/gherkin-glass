//! Policy: the suite's step sentences, grouped and sorted, with where each is used.

use std::cmp::Ordering;
use std::collections::BTreeMap;

use super::sentence_slots::sentence_of;
use crate::entities::{
    BlockHeading, ExamplesBlock, Sentence, SentenceGroup, SentenceUse, SiteFeaturePage, SiteNode,
    SiteTreeNode, StepKind, StepSentences, StepView, TableView,
};

/// The sentences of the steps of `pages`, feature pages of the site tree with their blocks.
pub(crate) fn find_step_sentences(pages: &[SiteFeaturePage<'_>]) -> StepSentences {
    let mut pages: Vec<_> = pages.to_vec();
    pages.sort_by(path_order);

    let mut found: BTreeMap<SentenceKey, Vec<SentenceUse>> = BTreeMap::new();
    for feature in pages {
        let page = feature.page();
        for FoundBlock {
            heading,
            columns,
            steps,
        } in blocks_of(&feature)
        {
            for step in steps {
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
                        heading: heading.parts.clone(),
                        line: step.line,
                        values,
                    });
            }
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

/// A background or a scenario: its heading, the columns of its outline, if it is one, and its
/// steps.
struct FoundBlock<'a> {
    heading: &'a BlockHeading,
    columns: Vec<&'a str>,
    steps: Vec<&'a StepView>,
}

/// Every background and scenario of the page, in file order; the scenarios an outline's rows
/// make are left out.
fn blocks_of<'a>(page: &SiteFeaturePage<'a>) -> Vec<FoundBlock<'a>> {
    page.blocks()
        .flat_map(with_rule_blocks)
        .filter_map(found_block)
        .collect()
}

/// `block`'s own blocks when it is a rule or the block of the scenarios outside every rule, or
/// else `block` itself.
fn with_rule_blocks(block: SiteTreeNode<'_>) -> Vec<SiteTreeNode<'_>> {
    match block.value() {
        SiteNode::Rule(_) | SiteNode::OutsideRules(_) => block.children().collect(),
        _ => vec![block],
    }
}

/// `block` with its steps, when it is a background or a scenario.
fn found_block(block: SiteTreeNode<'_>) -> Option<FoundBlock<'_>> {
    let heading = match block.value() {
        SiteNode::Background(background) => &background.heading,
        SiteNode::Scenario(scenario) => &scenario.heading,
        _ => return None,
    };
    Some(FoundBlock {
        heading,
        columns: columns_of(block),
        steps: block.steps().collect(),
    })
}

/// The names in the header rows of an outline's examples; none for a plain scenario.
fn columns_of(block: SiteTreeNode<'_>) -> Vec<&str> {
    block
        .examples()
        .filter_map(examples_table)
        .flat_map(header)
        .map(String::as_str)
        .collect()
}

fn examples_table(examples: &ExamplesBlock) -> Option<&TableView> {
    examples.table.as_ref()
}

fn header(table: &TableView) -> &[String] {
    &table.header
}

/// The order of the pages of `a` and `b`, by their paths, folder by folder.
fn path_order(a: &SiteFeaturePage<'_>, b: &SiteFeaturePage<'_>) -> Ordering {
    a.page().path.names().cmp(b.page().path.names())
}

#[cfg(test)]
mod tests;
