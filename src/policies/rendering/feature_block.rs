//! Policy: the blocks of a feature's page, each kind of block rendered through the one template of a block.

use askama::Template;

use super::node_rendering::{Render, rendered};
use crate::entities::{
    BackgroundBlock, BlockHeading, ExampleScenario, ExamplesAsScenarios, ExamplesBlock,
    FeatureBlock, FromBackground, HeadingLevel, HeadingPart, OutlineParts, Paragraph, RuleBlock,
    ScenarioBlock, StepPart, StepView, TableView, TagView,
};

#[derive(Template)]
#[template(path = "feature_block.html")]
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

impl Render for [FeatureBlock] {
    fn render(&self) -> String {
        joined(self.iter().map(FeatureBlock::render))
    }
}

impl Render for FeatureBlock {
    fn render(&self) -> String {
        match self {
            FeatureBlock::Background(background) => background.render(),
            FeatureBlock::Rule(rule) => rule.render(),
            FeatureBlock::Scenario(scenario) => scenario.render(),
        }
    }
}

impl Render for BackgroundBlock {
    fn render(&self) -> String {
        block(
            BlockView {
                description: &self.description,
                steps: &self.steps,
                ..BlockView::of(BlockKind::Background, &self.heading)
            },
            String::new(),
        )
    }
}

impl Render for RuleBlock {
    fn render(&self) -> String {
        let inner = self
            .background
            .iter()
            .map(BackgroundBlock::render)
            .chain(self.scenarios.iter().map(ScenarioBlock::render));
        block(
            BlockView {
                tags: &self.tags,
                description: &self.description,
                ..BlockView::of(BlockKind::Rule, &self.heading)
            },
            joined(inner),
        )
    }
}

impl Render for ScenarioBlock {
    fn render(&self) -> String {
        block(
            BlockView {
                tags: &self.tags,
                description: &self.description,
                backgrounds: &self.backgrounds,
                steps: &self.steps,
                ..BlockView::of(BlockKind::Scenario, &self.heading)
            },
            self.outline
                .as_ref()
                .map(OutlineParts::render)
                .unwrap_or_default(),
        )
    }
}

/// An outline's examples blocks, then its examples as scenarios.
impl Render for OutlineParts {
    fn render(&self) -> String {
        joined(
            self.examples
                .iter()
                .map(ExamplesBlock::render)
                .chain(self.as_scenarios.iter().map(ExamplesAsScenarios::render)),
        )
    }
}

impl Render for ExampleScenario {
    fn render(&self) -> String {
        block(
            BlockView {
                steps: &self.steps,
                ..BlockView::of(BlockKind::Scenario, &self.heading)
            },
            String::new(),
        )
    }
}

impl Render for ExamplesBlock {
    fn render(&self) -> String {
        block(
            BlockView {
                tags: &self.tags,
                description: &self.description,
                table: &self.table,
                ..BlockView::of(BlockKind::Examples, &self.heading)
            },
            String::new(),
        )
    }
}

impl Render for ExamplesAsScenarios {
    fn render(&self) -> String {
        block(
            BlockView::of(BlockKind::ExamplesAsScenarios, &self.heading),
            joined(self.scenarios.iter().map(ExampleScenario::render)),
        )
    }
}

/// `blocks` is what is under it, already rendered.
fn block(block: BlockView<'_>, blocks: String) -> String {
    rendered(&BlockTemplate { block, blocks })
}

/// Blocks one after the other, a line apart.
fn joined(blocks: impl Iterator<Item = String>) -> String {
    blocks.collect::<Vec<_>>().join("\n")
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
