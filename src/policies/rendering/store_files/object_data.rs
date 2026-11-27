//! Policy: an object of the store as JSON, with the hash of each of its blocks.

use serde::Serialize;

use super::{json_bytes, sha256_hex};
use crate::entities::{
    BlockKind, DocString, FeatureObject, ObjectBlock, ObjectRun, ObjectStep, StepKind, StepPart,
};

#[derive(Serialize)]
struct ObjectData<'a> {
    format: u32,
    language: &'a str,
    lang: &'a str,
    feature: BlockData<'a>,
}

#[derive(Serialize)]
struct BlockData<'a> {
    kind: &'static str,
    keyword: &'a str,
    note: bool,
    name: &'a str,
    description: &'a str,
    tags: &'a [String],
    anchor: &'a str,
    hash: String,
    blocks: Vec<BlockData<'a>>,
    steps: Vec<StepData<'a>>,
    examples: Vec<BlockData<'a>>,
    place: Option<usize>,
    table: Option<&'a [Vec<String>]>,
    runs: Vec<RunData<'a>>,
}

#[derive(Serialize)]
struct StepData<'a> {
    keyword: &'a str,
    text: &'a str,
    kind: &'static str,
    sentence: &'a str,
    slots: &'a [String],
    parts: Vec<PartData>,
    table: Option<&'a [Vec<String>]>,
    doc_string: Option<DocStringData<'a>>,
}

#[derive(Serialize)]
struct PartData {
    kind: &'static str,
    text: String,
    parts: Vec<PartData>,
}

#[derive(Serialize)]
struct DocStringData<'a> {
    media_type: Option<&'a str>,
    content: &'a str,
}

#[derive(Serialize)]
struct RunData<'a> {
    anchor: &'a str,
    steps: Vec<StepData<'a>>,
}

/// A step as its block's hash takes it: its keyword, its text, its table and its doc string.
type HashedStep<'a> = (
    &'a str,
    &'a str,
    Option<&'a [Vec<String>]>,
    Option<DocStringData<'a>>,
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

/// The compact JSON of `object`.
pub(super) fn object_json(object: &FeatureObject) -> Vec<u8> {
    let feature = block_data(&object.feature, Some(&object.language));
    json_bytes(&ObjectData {
        format: super::STORE_FORMAT,
        language: &object.language,
        lang: &object.lang,
        feature,
    })
}

/// `language` is the feature's, which its hash adds, and `None` for every other block.
fn block_data<'a>(block: &'a ObjectBlock, language: Option<&'a str>) -> BlockData<'a> {
    BlockData {
        kind: block.kind.as_str(),
        keyword: &block.keyword,
        note: block.note,
        name: &block.name,
        description: &block.description,
        tags: &block.tags,
        anchor: &block.anchor,
        hash: block_hash(block, language),
        blocks: block.blocks.iter().map(inner_block_data).collect(),
        steps: block.steps.iter().map(step_data).collect(),
        examples: block.examples.iter().map(inner_block_data).collect(),
        place: block.place,
        table: block.table.as_deref(),
        runs: block.runs.iter().map(run_data).collect(),
    }
}

fn inner_block_data(block: &ObjectBlock) -> BlockData<'_> {
    block_data(block, None)
}

fn step_data(step: &ObjectStep) -> StepData<'_> {
    StepData {
        keyword: &step.keyword,
        text: &step.text,
        kind: step_kind(step.kind),
        sentence: &step.sentence,
        slots: &step.slots,
        parts: step.parts.iter().map(part_data).collect(),
        table: step.table.as_deref(),
        doc_string: step.doc_string.as_ref().map(doc_string_data),
    }
}

fn step_kind(kind: StepKind) -> &'static str {
    match kind {
        StepKind::Given => "given",
        StepKind::When => "when",
        StepKind::Then => "then",
    }
}

fn part_data(part: &StepPart) -> PartData {
    let (kind, parts) = match part {
        StepPart::Plain(_) => ("text", Vec::new()),
        StepPart::Number(_) => ("number", Vec::new()),
        StepPart::Placeholder(_) => ("placeholder", Vec::new()),
        StepPart::Parameter(inside) => ("parameter", inside.iter().map(part_data).collect()),
    };
    PartData {
        kind,
        text: part.text(),
        parts,
    }
}

fn doc_string_data(doc_string: &DocString) -> DocStringData<'_> {
    DocStringData {
        media_type: doc_string.media_type.as_deref(),
        content: &doc_string.content,
    }
}

fn run_data(run: &ObjectRun) -> RunData<'_> {
    RunData {
        anchor: &run.anchor,
        steps: run.steps.iter().map(step_data).collect(),
    }
}

/// The SHA-256 of the compact JSON of what `block`'s page shows of it, as rule 251 lists it:
/// its kind, its keyword, its name, its description, its tags and its steps, then what its
/// kind adds, the feature its `language`.
fn block_hash(block: &ObjectBlock, language: Option<&str>) -> String {
    let bytes = match (block.kind, language) {
        (BlockKind::Feature, Some(language)) => hashed_with(block, language),
        (BlockKind::Examples, _) => hashed_with(block, block.table.as_deref()),
        (BlockKind::ScenarioOutline, _) => {
            let examples: Vec<_> = block.examples.iter().map(hashed_examples).collect();
            hashed_with(block, examples)
        }
        _ => {
            let (kind, keyword, name, description, tags, steps) = hashed_parts(block);
            json_bytes(&(kind, keyword, name, description, tags, steps))
        }
    };
    sha256_hex(&bytes)
}

/// The list of what `block`'s hash is made from, ending with `added`, what its kind adds.
fn hashed_with(block: &ObjectBlock, added: impl Serialize) -> Vec<u8> {
    let (kind, keyword, name, description, tags, steps) = hashed_parts(block);
    json_bytes(&(kind, keyword, name, description, tags, steps, added))
}

/// What every block's hash is made from.
fn hashed_parts(
    block: &ObjectBlock,
) -> (
    &'static str,
    &str,
    &str,
    &str,
    &[String],
    Vec<HashedStep<'_>>,
) {
    (
        block.kind.as_str(),
        &block.keyword,
        &block.name,
        &block.description,
        &block.tags,
        block.steps.iter().map(hashed_step).collect(),
    )
}

fn hashed_step(step: &ObjectStep) -> HashedStep<'_> {
    (
        &step.keyword,
        &step.text,
        step.table.as_deref(),
        step.doc_string.as_ref().map(doc_string_data),
    )
}

fn hashed_examples(examples: &ObjectBlock) -> HashedExamples<'_> {
    (
        &examples.keyword,
        &examples.name,
        &examples.description,
        &examples.tags,
        examples.table.as_deref(),
    )
}
