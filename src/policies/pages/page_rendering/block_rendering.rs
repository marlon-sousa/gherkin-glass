//! Policy: the HTML of a feature page's blocks, each kind through the one template of a block.

use askama::Template;

use super::render;
use crate::entities::{
    BackgroundBlock, BlockHeading, ExampleScenario, ExamplesAsScenarios, ExamplesBlock,
    FeatureBlock, FromBackground, HeadingLevel, HeadingPart, OutlineParts, Paragraph, RuleBlock,
    ScenarioBlock, StepPart, StepView, TableView, TagView,
};

#[derive(Template)]
#[template(path = "block.html")]
struct BlockTemplate<'a> {
    block: BlockView<'a>,
    blocks: String,
}

/// What the template of a block shows, whatever its kind; a kind leaves
/// empty what it cannot have.
struct BlockView<'a> {
    kind: BlockKind,
    level: HeadingLevel,
    anchor: &'a str,
    heading_parts: &'a [HeadingPart],
    collapsed: bool,
    tags: &'a [TagView],
    description: &'a Option<Paragraph>,
    backgrounds: &'a [FromBackground],
    steps: &'a [StepView],
    table: &'a Option<TableView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockKind {
    Background,
    Rule,
    Scenario,
    Examples,
    ExamplesAsScenarios,
}

pub(super) fn render_blocks(blocks: &[FeatureBlock]) -> String {
    blocks
        .iter()
        .map(render_feature_block)
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_feature_block(block: &FeatureBlock) -> String {
    match block {
        FeatureBlock::Background(background) => render_background(background),
        FeatureBlock::Rule(rule) => render_rule(rule),
        FeatureBlock::Scenario(scenario) => render_scenario(scenario),
    }
}

fn render_background(background: &BackgroundBlock) -> String {
    render_block(
        BlockView {
            description: &background.description,
            steps: &background.steps,
            ..BlockView::of(BlockKind::Background, &background.heading)
        },
        String::new(),
    )
}

fn render_rule(rule: &RuleBlock) -> String {
    let inner: Vec<_> = rule
        .background
        .iter()
        .map(render_background)
        .chain(rule.scenarios.iter().map(render_scenario))
        .collect();
    render_block(
        BlockView {
            tags: &rule.tags,
            description: &rule.description,
            ..BlockView::of(BlockKind::Rule, &rule.heading)
        },
        inner.join("\n"),
    )
}

fn render_scenario(scenario: &ScenarioBlock) -> String {
    let inner = scenario
        .outline
        .as_ref()
        .map(render_outline)
        .unwrap_or_default();
    render_block(
        BlockView {
            tags: &scenario.tags,
            description: &scenario.description,
            backgrounds: &scenario.backgrounds,
            steps: &scenario.steps,
            ..BlockView::of(BlockKind::Scenario, &scenario.heading)
        },
        inner,
    )
}

/// An outline's examples blocks, then its examples as scenarios.
fn render_outline(outline: &OutlineParts) -> String {
    let inner: Vec<_> = outline
        .examples
        .iter()
        .map(render_examples)
        .chain(outline.as_scenarios.iter().map(render_as_scenarios))
        .collect();
    inner.join("\n")
}

fn render_example_scenario(scenario: &ExampleScenario) -> String {
    render_block(
        BlockView {
            steps: &scenario.steps,
            ..BlockView::of(BlockKind::Scenario, &scenario.heading)
        },
        String::new(),
    )
}

fn render_examples(examples: &ExamplesBlock) -> String {
    render_block(
        BlockView {
            tags: &examples.tags,
            description: &examples.description,
            table: &examples.table,
            ..BlockView::of(BlockKind::Examples, &examples.heading)
        },
        String::new(),
    )
}

fn render_as_scenarios(examples: &ExamplesAsScenarios) -> String {
    let inner: Vec<_> = examples
        .scenarios
        .iter()
        .map(render_example_scenario)
        .collect();
    render_block(
        BlockView::of(BlockKind::ExamplesAsScenarios, &examples.heading),
        inner.join("\n"),
    )
}

/// `blocks` is what is under it, already rendered.
fn render_block(block: BlockView<'_>, blocks: String) -> String {
    render(&BlockTemplate { block, blocks })
}

impl<'a> BlockView<'a> {
    /// A block of `kind` with only its heading.
    fn of(kind: BlockKind, heading: &'a BlockHeading) -> Self {
        BlockView {
            kind,
            level: heading.level,
            anchor: &heading.anchor,
            heading_parts: &heading.parts,
            collapsed: heading.collapsed,
            tags: &[],
            description: &None,
            backgrounds: &[],
            steps: &[],
            table: &None,
        }
    }
}

impl BlockKind {
    /// The class the stylesheet and the script know the block by.
    fn class(self) -> &'static str {
        match self {
            BlockKind::Background => "ghg-background",
            BlockKind::Rule => "ghg-rule",
            BlockKind::Scenario => "ghg-scenario",
            BlockKind::Examples => "ghg-examples",
            BlockKind::ExamplesAsScenarios => "ghg-examples-as-scenarios",
        }
    }
}
