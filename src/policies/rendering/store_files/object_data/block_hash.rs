//! Policy: a block's hash, the SHA-256 of the compact JSON of a list of what its page shows of it: its kind, its keyword, its name, its description, its tags and its steps, then what its kind adds.

use serde::Serialize;

use super::{DocStringData, doc_string_data};
use crate::entities::{
    BlockContent, BlockKind, BlockText, FeatureObject, ObjectBlock, ObjectExamples, ObjectStep,
};
use crate::policies::rendering::{json_bytes, sha256_hex};

/// A step as its block's hash takes it: its keyword, its text, its table and its doc string.
type HashedStep<'a> = (
    &'a str,
    &'a str,
    Option<&'a [Vec<String>]>,
    Option<DocStringData<'a>>,
);

/// What every block's hash starts with: its kind, keyword, name, description, tags and steps.
type HashedParts<'a> = (
    &'static str,
    &'a str,
    &'a str,
    &'a str,
    &'a [String],
    Vec<HashedStep<'a>>,
);

/// An outline's examples block as the outline's hash takes it: its keyword, its name, its
/// description, its tags and its table.
type HashedExamples<'a> = (
    &'a str,
    &'a str,
    &'a str,
    &'a [String],
    Option<&'a [Vec<String>]>,
);

/// The hash of a block below the feature: a background's, a rule's or a scenario's ends with
/// its steps, a rule's none; an outline's adds its examples blocks.
pub(super) fn block_hash(block: &ObjectBlock) -> String {
    let kind = block.kind();
    let bytes = match &block.content {
        BlockContent::Background { steps } | BlockContent::Scenario { steps } => {
            json_bytes(&parts(kind, &block.text, steps))
        }
        BlockContent::Rule { .. } => json_bytes(&parts(kind, &block.text, &[])),
        BlockContent::Outline { steps, examples } => {
            let examples: Vec<_> = examples.iter().map(hashed_examples).collect();
            json_bytes(&with(parts(kind, &block.text, steps), examples))
        }
    };
    sha256_hex(&bytes)
}

/// The feature's hash, which adds its language.
pub(super) fn feature_hash(object: &FeatureObject) -> String {
    let parts = parts(BlockKind::Feature, &object.text, &[]);
    sha256_hex(&json_bytes(&with(parts, &object.language)))
}

/// An examples block's hash, which adds its table.
pub(super) fn examples_hash(examples: &ObjectExamples) -> String {
    let parts = parts(BlockKind::Examples, &examples.text, &[]);
    sha256_hex(&json_bytes(&with(parts, examples.table.as_deref())))
}

fn parts<'a>(kind: BlockKind, text: &'a BlockText, steps: &'a [ObjectStep]) -> HashedParts<'a> {
    (
        kind.as_str(),
        &text.keyword,
        &text.name,
        &text.description,
        &text.tags,
        steps.iter().map(hashed_step).collect(),
    )
}

/// `parts` followed by `added`, as one flat list.
fn with<T: Serialize>(
    (kind, keyword, name, description, tags, steps): HashedParts<'_>,
    added: T,
) -> impl Serialize {
    (kind, keyword, name, description, tags, steps, added)
}

fn hashed_step(step: &ObjectStep) -> HashedStep<'_> {
    (
        &step.keyword,
        &step.text,
        step.table.as_deref(),
        step.doc_string.as_ref().map(doc_string_data),
    )
}

fn hashed_examples(examples: &ObjectExamples) -> HashedExamples<'_> {
    (
        &examples.text.keyword,
        &examples.text.name,
        &examples.text.description,
        &examples.text.tags,
        examples.table.as_deref(),
    )
}

#[cfg(test)]
mod tests;
