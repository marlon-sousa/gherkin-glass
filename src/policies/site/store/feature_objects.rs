//! Policy: a feature file's object, its blocks with their anchors and its steps with what ghg works out of them, and the line of each block's keyword, which its descriptor holds.

mod object_steps;

use gherkin::{Background, Examples, Rule, Scenario};

use crate::entities::{
    Anchored, AnchoredRule, AnchoredScenario, BlockContent, BlockKind, BlockText, FeatureObject,
    ObjectBlock, ObjectExamples, ParsedFeature,
};
use crate::policies::site::{
    anchor_feature, block_rows, description_lines, page_language, writes_another_keyword,
};

use object_steps::ObjectSteps;

/// A block of the object, and the line of its keyword followed by those of the blocks inside
/// it, as the descriptor lists them.
struct Built<T> {
    block: T,
    lines: Vec<usize>,
}

/// `parsed`'s object, and the line of each of its blocks' keywords: the feature's, then every
/// block inside it, depth first in file order, each followed by its blocks, then its examples.
pub(super) fn feature_object(parsed: &ParsedFeature) -> (FeatureObject, Vec<usize>) {
    let feature = &parsed.feature;
    let anchored = anchor_feature(parsed);
    let builder = ObjectBuilder {
        steps: ObjectSteps { parsed },
    };
    let background = anchored
        .background
        .iter()
        .map(|background| builder.background(background));
    let scenarios = anchored
        .scenarios
        .iter()
        .map(|scenario| builder.scenario(scenario));
    let rules = anchored.rules.iter().map(|rule| builder.rule(rule));
    let (blocks, lines) = split_all(background.chain(scenarios).chain(rules));
    let object = FeatureObject {
        language: parsed.language.clone(),
        lang: page_language(&parsed.language).to_string(),
        text: block_text(
            BlockKind::Feature,
            &feature.keyword,
            &feature.name,
            feature.description.as_deref(),
            tagged(&feature.tags),
        ),
        blocks,
    };
    (object, followed(feature.position.line, lines))
}

/// Builds a feature's blocks, with the steps `steps` builds.
struct ObjectBuilder<'a> {
    steps: ObjectSteps<'a>,
}

impl ObjectBuilder<'_> {
    fn background(&self, anchored: &Anchored<'_, Background>) -> Built<ObjectBlock> {
        let background = anchored.block;
        Built {
            block: ObjectBlock {
                text: block_text(
                    BlockKind::Background,
                    &background.keyword,
                    &background.name,
                    background.description.as_deref(),
                    Vec::new(),
                ),
                anchor: anchored.anchor.clone(),
                content: BlockContent::Background {
                    steps: self.steps.steps(&background.steps, None),
                },
            },
            lines: vec![background.position.line],
        }
    }

    fn rule(&self, anchored: &AnchoredRule<'_>) -> Built<ObjectBlock> {
        let rule: &Rule = anchored.rule;
        let background = anchored
            .background
            .iter()
            .map(|background| self.background(background));
        let scenarios = anchored
            .scenarios
            .iter()
            .map(|scenario| self.scenario(scenario));
        let (blocks, lines) = split_all(background.chain(scenarios));
        Built {
            block: ObjectBlock {
                text: block_text(
                    BlockKind::Rule,
                    &rule.keyword,
                    &rule.name,
                    rule.description.as_deref(),
                    tagged(&rule.tags),
                ),
                anchor: anchored.anchor.clone(),
                content: BlockContent::Rule { blocks },
            },
            lines: followed(rule.position.line, lines),
        }
    }

    fn scenario(&self, anchored: &AnchoredScenario<'_>) -> Built<ObjectBlock> {
        let scenario = anchored.scenario;
        let steps = self.steps.steps(&scenario.steps, Some(scenario));
        let mut row_anchors = anchored.rows.iter();
        let (examples, lines) =
            split_all(anchored.examples.iter().zip(1..).map(|(examples, place)| {
                self.examples(scenario, examples, place, &mut row_anchors)
            }));
        let (kind, content) = if examples.is_empty() {
            (BlockKind::Scenario, BlockContent::Scenario { steps })
        } else {
            (
                BlockKind::ScenarioOutline,
                BlockContent::Outline { steps, examples },
            )
        };
        Built {
            block: ObjectBlock {
                text: block_text(
                    kind,
                    &scenario.keyword,
                    &scenario.name,
                    scenario.description.as_deref(),
                    tagged(&scenario.tags),
                ),
                anchor: anchored.anchor.clone(),
                content,
            },
            lines: followed(scenario.position.line, lines),
        }
    }

    /// The examples block at `place` among `outline`'s, whose rows take the next of
    /// `row_anchors`, which number the rows across every examples block of the outline.
    fn examples<'a>(
        &self,
        outline: &Scenario,
        anchored: &Anchored<'_, Examples>,
        place: usize,
        row_anchors: &mut impl Iterator<Item = &'a String>,
    ) -> Built<ObjectExamples> {
        let examples = anchored.block;
        let runs = block_rows(examples)
            .iter()
            .zip(row_anchors)
            .map(|(row, anchor)| self.steps.run(outline, row, anchor))
            .collect();
        Built {
            block: ObjectExamples {
                text: block_text(
                    BlockKind::Examples,
                    &examples.keyword,
                    examples.name.as_deref().unwrap_or_default(),
                    examples.description.as_deref(),
                    tagged(&examples.tags),
                ),
                anchor: anchored.anchor.clone(),
                place,
                table: examples.table.as_ref().map(|table| table.rows.clone()),
                runs,
            },
            lines: vec![examples.position.line],
        }
    }
}

/// The blocks of `built`, and their lines, one after the other.
fn split_all<T>(built: impl Iterator<Item = Built<T>>) -> (Vec<T>, Vec<usize>) {
    let (blocks, lines): (Vec<T>, Vec<Vec<usize>>) = built.map(split).unzip();
    (blocks, lines.concat())
}

fn split<T>(built: Built<T>) -> (T, Vec<usize>) {
    (built.block, built.lines)
}

/// `line`, followed by `lines`.
fn followed(line: usize, lines: Vec<usize>) -> Vec<usize> {
    [vec![line], lines].concat()
}

fn block_text(
    kind: BlockKind,
    keyword: &str,
    name: &str,
    description: Option<&str>,
    tags: Vec<String>,
) -> BlockText {
    BlockText {
        keyword: keyword.to_string(),
        note: writes_another_keyword(kind, keyword),
        name: name.to_string(),
        description: description_lines(description).join("\n"),
        tags,
    }
}

/// Each with its `@`, as the `gherkin` crate keeps them without it.
fn tagged(tags: &[String]) -> Vec<String> {
    tags.iter().map(at_tag).collect()
}

fn at_tag(tag: &String) -> String {
    format!("@{tag}")
}

#[cfg(test)]
mod tests;
