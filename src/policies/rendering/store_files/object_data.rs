//! Policy: an object of the store as JSON, every block written with every member, with the hash of each.

mod block_hash;

use serde::Serialize;

use crate::entities::{
    BlockContent, BlockKind, BlockText, DocString, FeatureObject, ObjectBlock, ObjectExamples,
    ObjectRun, ObjectStep, StepKind, StepPart,
};
use crate::policies::rendering::{STORE_FORMAT, json_bytes};
use block_hash::{block_hash, examples_hash, feature_hash};

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
pub(super) struct StepData<'a> {
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
pub(super) struct DocStringData<'a> {
    media_type: Option<&'a str>,
    content: &'a str,
}

#[derive(Serialize)]
struct RunData<'a> {
    anchor: &'a str,
    steps: Vec<StepData<'a>>,
}

/// The compact JSON of `object`.
pub(super) fn object_json(object: &FeatureObject) -> Vec<u8> {
    let feature = BlockData {
        hash: feature_hash(object),
        blocks: object.blocks.iter().map(block_data).collect(),
        ..empty_block(BlockKind::Feature, &object.text, "")
    };
    json_bytes(&ObjectData {
        format: STORE_FORMAT,
        language: &object.language,
        lang: &object.lang,
        feature,
    })
}

fn block_data(block: &ObjectBlock) -> BlockData<'_> {
    let empty = empty_block(block.kind(), &block.text, &block.anchor);
    let hash = block_hash(block);
    match &block.content {
        BlockContent::Background { steps } | BlockContent::Scenario { steps } => BlockData {
            hash,
            steps: steps.iter().map(step_data).collect(),
            ..empty
        },
        BlockContent::Rule { blocks } => BlockData {
            hash,
            blocks: blocks.iter().map(block_data).collect(),
            ..empty
        },
        BlockContent::Outline { steps, examples } => BlockData {
            hash,
            steps: steps.iter().map(step_data).collect(),
            examples: examples.iter().map(examples_data).collect(),
            ..empty
        },
    }
}

fn examples_data(examples: &ObjectExamples) -> BlockData<'_> {
    BlockData {
        hash: examples_hash(examples),
        place: Some(examples.place),
        table: examples.table.as_deref(),
        runs: examples.runs.iter().map(run_data).collect(),
        ..empty_block(BlockKind::Examples, &examples.text, &examples.anchor)
    }
}

/// A block of `kind` with `text` and `anchor`, holding nothing, its hash still to be made.
fn empty_block<'a>(kind: BlockKind, text: &'a BlockText, anchor: &'a str) -> BlockData<'a> {
    BlockData {
        kind: kind.as_str(),
        keyword: &text.keyword,
        note: text.note,
        name: &text.name,
        description: &text.description,
        tags: &text.tags,
        anchor,
        hash: String::new(),
        blocks: Vec::new(),
        steps: Vec::new(),
        examples: Vec::new(),
        place: None,
        table: None,
        runs: Vec::new(),
    }
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

pub(super) fn doc_string_data(doc_string: &DocString) -> DocStringData<'_> {
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
