//! Policy: a feature file's object, its blocks with their anchors and its steps with what ghg works out of them, and the line of each block's keyword, which its descriptor holds.

use gherkin::{Background, Examples, Feature, Scenario, Step, Table};

use crate::entities::{
    Anchored, AnchoredRule, AnchoredScenario, BlockKind, DocString, FeatureObject, ObjectBlock,
    ObjectRun, ObjectStep, ParsedFeature,
};
use crate::policies::site::{
    ExampleRow, anchor_feature, block_rows, highlight_step, page_language, sentence_of, step_kind,
    writes_another_keyword,
};

/// A block of the object, and the line of its keyword followed by those of the blocks inside
/// it, as the descriptor lists them.
struct Built {
    block: ObjectBlock,
    lines: Vec<usize>,
}

/// `parsed`'s object, and the line of each of its blocks' keywords: the feature's, then every
/// block inside it, depth first in file order, each followed by its blocks, then its examples.
pub(super) fn feature_object(parsed: &ParsedFeature) -> (FeatureObject, Vec<usize>) {
    let feature = &parsed.feature;
    let anchored = anchor_feature(parsed);
    let steps = ObjectSteps { parsed };
    let background = anchored
        .background
        .iter()
        .map(|background| steps.background(background));
    let scenarios = anchored
        .scenarios
        .iter()
        .map(|scenario| steps.scenario(scenario));
    let rules = anchored.rules.iter().map(|rule| steps.rule(rule));
    let inside: Vec<Built> = background.chain(scenarios).chain(rules).collect();
    let built = holding(
        feature_block(feature),
        feature.position.line,
        inside,
        Vec::new(),
    );
    let object = FeatureObject {
        language: parsed.language.clone(),
        lang: page_language(&parsed.language).to_string(),
        feature: built.block,
    };
    (object, built.lines)
}

/// Builds a feature's blocks and steps, which need its doc strings' media types.
struct ObjectSteps<'a> {
    parsed: &'a ParsedFeature,
}

impl ObjectSteps<'_> {
    fn background(&self, anchored: &Anchored<'_, Background>) -> Built {
        let background = anchored.block;
        let block = ObjectBlock {
            steps: self.steps(&background.steps, None),
            ..block_of(
                BlockKind::Background,
                &background.keyword,
                &background.name,
                background.description.as_deref(),
                &anchored.anchor,
            )
        };
        holding(block, background.position.line, Vec::new(), Vec::new())
    }

    fn rule(&self, anchored: &AnchoredRule<'_>) -> Built {
        let rule = anchored.rule;
        let background = anchored
            .background
            .iter()
            .map(|background| self.background(background));
        let scenarios = anchored
            .scenarios
            .iter()
            .map(|scenario| self.scenario(scenario));
        let block = ObjectBlock {
            tags: tagged(&rule.tags),
            ..block_of(
                BlockKind::Rule,
                &rule.keyword,
                &rule.name,
                rule.description.as_deref(),
                &anchored.anchor,
            )
        };
        holding(
            block,
            rule.position.line,
            background.chain(scenarios).collect(),
            Vec::new(),
        )
    }

    fn scenario(&self, anchored: &AnchoredScenario<'_>) -> Built {
        let scenario = anchored.scenario;
        let kind = if scenario.examples.is_empty() {
            BlockKind::Scenario
        } else {
            BlockKind::ScenarioOutline
        };
        let mut row_anchors = anchored.rows.iter();
        let examples = anchored
            .examples
            .iter()
            .zip(1..)
            .map(|(examples, place)| self.examples(scenario, examples, place, &mut row_anchors))
            .collect();
        let block = ObjectBlock {
            tags: tagged(&scenario.tags),
            steps: self.steps(&scenario.steps, Some(scenario)),
            ..block_of(
                kind,
                &scenario.keyword,
                &scenario.name,
                scenario.description.as_deref(),
                &anchored.anchor,
            )
        };
        holding(block, scenario.position.line, Vec::new(), examples)
    }

    /// The examples block at `place` among `outline`'s, whose rows take the next of
    /// `row_anchors`, which number the rows across every examples block of the outline.
    fn examples<'a>(
        &self,
        outline: &Scenario,
        anchored: &Anchored<'_, Examples>,
        place: usize,
        row_anchors: &mut impl Iterator<Item = &'a String>,
    ) -> Built {
        let examples = anchored.block;
        let runs = block_rows(examples)
            .iter()
            .zip(row_anchors)
            .map(|(row, anchor)| self.run(outline, row, anchor))
            .collect();
        let block = ObjectBlock {
            tags: tagged(&examples.tags),
            place: Some(place),
            table: examples.table.as_ref().map(rows_of),
            runs,
            ..block_of(
                BlockKind::Examples,
                &examples.keyword,
                examples.name.as_deref().unwrap_or_default(),
                examples.description.as_deref(),
                &anchored.anchor,
            )
        };
        holding(block, examples.position.line, Vec::new(), Vec::new())
    }

    /// The scenario `row` of `outline`'s examples makes, at `anchor`.
    fn run(&self, outline: &Scenario, row: &ExampleRow<'_>, anchor: &str) -> ObjectRun {
        ObjectRun {
            anchor: anchor.to_string(),
            steps: self.filled_steps(&outline.steps, row),
        }
    }

    /// `scenario` holds the steps, or is `None` for a background's.
    fn steps(&self, steps: &[Step], scenario: Option<&Scenario>) -> Vec<ObjectStep> {
        let columns = columns_of(scenario);
        steps
            .iter()
            .map(|step| self.step(step, scenario, &columns))
            .collect()
    }

    fn step(&self, step: &Step, scenario: Option<&Scenario>, columns: &[&str]) -> ObjectStep {
        let (sentence, slots) = sentence_of(&step.value, columns);
        ObjectStep {
            keyword: step.keyword.clone(),
            text: step.value.clone(),
            kind: step_kind(step.ty),
            sentence,
            slots,
            parts: highlight_step(step, scenario).parts,
            table: step.table.as_ref().map(rows_of),
            doc_string: self.doc_string(step),
        }
    }

    fn doc_string(&self, step: &Step) -> Option<DocString> {
        Some(DocString {
            media_type: self.parsed.media_type(step).map(str::to_string),
            content: step.docstring.clone()?,
        })
    }

    /// The outline's steps with `row`'s values in place of their placeholders, in their text,
    /// their tables and their doc strings, so that none of them is a placeholder any more.
    fn filled_steps(&self, steps: &[Step], row: &ExampleRow<'_>) -> Vec<ObjectStep> {
        steps
            .iter()
            .map(|step| self.filled_step(step, row))
            .collect()
    }

    fn filled_step(&self, step: &Step, row: &ExampleRow<'_>) -> ObjectStep {
        let filled = Step {
            value: row.fill(&step.value),
            ..step.clone()
        };
        ObjectStep {
            table: step.table.as_ref().map(|table| filled_rows(table, row)),
            doc_string: self
                .doc_string(step)
                .map(|doc_string| filled_doc_string(doc_string, row)),
            ..self.step(&filled, None, &[])
        }
    }
}

/// `block` with the line of its keyword, holding `inside` and `examples`, whose lines follow
/// it in that order.
fn holding(mut block: ObjectBlock, line: usize, inside: Vec<Built>, examples: Vec<Built>) -> Built {
    let mut lines = vec![line];
    let (blocks, inside_lines): (Vec<_>, Vec<_>) = inside.into_iter().map(split).unzip();
    let (examples, examples_lines): (Vec<_>, Vec<_>) = examples.into_iter().map(split).unzip();
    lines.extend(inside_lines.into_iter().flatten());
    lines.extend(examples_lines.into_iter().flatten());
    block.blocks = blocks;
    block.examples = examples;
    Built { block, lines }
}

fn split(built: Built) -> (ObjectBlock, Vec<usize>) {
    (built.block, built.lines)
}

fn feature_block(feature: &Feature) -> ObjectBlock {
    ObjectBlock {
        tags: tagged(&feature.tags),
        ..block_of(
            BlockKind::Feature,
            &feature.keyword,
            &feature.name,
            feature.description.as_deref(),
            "",
        )
    }
}

/// A block of `kind` with nothing inside it, no tags and no examples' parts.
fn block_of(
    kind: BlockKind,
    keyword: &str,
    name: &str,
    description: Option<&str>,
    anchor: &str,
) -> ObjectBlock {
    ObjectBlock {
        kind,
        keyword: keyword.to_string(),
        note: writes_another_keyword(kind, keyword),
        name: name.to_string(),
        description: description_of(description),
        tags: Vec::new(),
        anchor: anchor.to_string(),
        blocks: Vec::new(),
        steps: Vec::new(),
        examples: Vec::new(),
        place: None,
        table: None,
        runs: Vec::new(),
    }
}

/// The description's lines that hold text, without the spaces at their ends, one to a line,
/// as its page shows them; empty for none.
fn description_of(text: Option<&str>) -> String {
    text.unwrap_or_default()
        .lines()
        .map(str::trim_end)
        .filter(has_text)
        .collect::<Vec<_>>()
        .join("\n")
}

fn has_text(line: &&str) -> bool {
    !line.trim().is_empty()
}

/// Each with its `@`, as the `gherkin` crate keeps them without it.
fn tagged(tags: &[String]) -> Vec<String> {
    tags.iter().map(|tag| format!("@{tag}")).collect()
}

fn rows_of(table: &Table) -> Vec<Vec<String>> {
    table.rows.clone()
}

fn filled_doc_string(doc_string: DocString, row: &ExampleRow<'_>) -> DocString {
    DocString {
        media_type: doc_string.media_type.map(|text| row.fill(&text)),
        content: row.fill(&doc_string.content),
    }
}

fn filled_rows(table: &Table, row: &ExampleRow<'_>) -> Vec<Vec<String>> {
    table
        .rows
        .iter()
        .map(|cells| filled_cells(cells, row))
        .collect()
}

fn filled_cells(cells: &[String], row: &ExampleRow<'_>) -> Vec<String> {
    cells.iter().map(|cell| row.fill(cell)).collect()
}

/// The names in the header rows of an outline's examples; none for a plain scenario or a
/// background.
fn columns_of(scenario: Option<&Scenario>) -> Vec<&str> {
    scenario
        .into_iter()
        .flat_map(|scenario| &scenario.examples)
        .filter_map(|examples| examples.table.as_ref())
        .filter_map(|table| table.rows.first())
        .flatten()
        .map(String::as_str)
        .collect()
}

#[cfg(test)]
mod tests;
