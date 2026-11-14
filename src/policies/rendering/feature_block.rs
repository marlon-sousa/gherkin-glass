//! Policy: the blocks of a feature's page, each kind of block node rendered through the rendering trait into the one template of a block, with the backgrounds a scenario starts with, each a block of its own, its steps, an outline's list of examples and the blocks below it.

mod scenario_backgrounds;

use askama::Template;
use scenario_backgrounds::{backgrounds_in_scenario, steps_heading};

use super::examples_list::{examples_list, is_examples};
use super::node_rendering::{NodeOf, Render, pieces, rendered};
use super::steps::{step_list, steps_of};
use crate::entities::{
    BackgroundBlock, BlockHeading, HeadingLevel, HeadingPart, Paragraph, RuleBlock, ScenarioBlock,
    SiteTreeNode, TagView,
};

#[derive(Template)]
#[template(path = "feature_block.html")]
struct BlockTemplate<'a> {
    block: BlockView<'a>,
    /// The blocks of the backgrounds a scenario starts with, already rendered.
    backgrounds: Vec<String>,
    /// The heading of a scenario's own steps, after the backgrounds it holds;
    /// `None` when it holds none or has no steps of its own.
    steps_heading: Option<StepsHeading<'a>>,
    /// The block's own steps, already rendered; empty for a block with none.
    steps: String,
    /// An outline's list of examples, already rendered; empty for any other block.
    examples: String,
    /// The blocks below it, already rendered.
    blocks: String,
}

/// What the template of a block shows of its own node, whatever its kind; a
/// kind leaves empty what it cannot have.
struct BlockView<'a> {
    kind: BlockKind,
    level: HeadingLevel,
    anchor: &'a str,
    heading_parts: &'a [HeadingPart],
    collapsed: bool,
    /// For a background whose state the page's checkbox sets, whether it is
    /// collapsed while the box is unchecked; `None` for any other block.
    collapsed_when_unchecked: Option<bool>,
    tags: &'a [TagView],
    description: &'a Option<Paragraph>,
}

/// A heading with no button.
struct StepsHeading<'a> {
    level: HeadingLevel,
    text: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BlockKind {
    Background,
    /// A background inside a scenario that starts with its steps.
    BackgroundInScenario,
    Rule,
    /// The block of the scenarios outside every rule.
    OutsideRules,
    Scenario,
    ExamplesAsScenarios,
}

/// A block known by its heading alone, of `kind`: the block of the scenarios
/// outside every rule, which holds them, an outline's examples as scenarios,
/// which holds the scenario of each row, or the scenario of one row, which
/// holds its steps; with its node.
pub(super) struct HeadedNode<'a> {
    pub(super) kind: BlockKind,
    pub(super) heading: &'a BlockHeading,
    pub(super) node: SiteTreeNode<'a>,
}

impl Render for NodeOf<'_, BackgroundBlock> {
    fn render(&self) -> String {
        let view = BlockView {
            description: &self.value.description,
            collapsed_when_unchecked: self.value.collapsed_when_unchecked,
            ..BlockView::of(BlockKind::Background, &self.value.heading)
        };
        block(view, self.node)
    }
}

impl Render for NodeOf<'_, RuleBlock> {
    fn render(&self) -> String {
        let view = BlockView {
            tags: &self.value.tags,
            description: &self.value.description,
            ..BlockView::of(BlockKind::Rule, &self.value.heading)
        };
        block(view, self.node)
    }
}

impl Render for NodeOf<'_, ScenarioBlock> {
    fn render(&self) -> String {
        let view = BlockView {
            tags: &self.value.tags,
            description: &self.value.description,
            ..BlockView::of(BlockKind::Scenario, &self.value.heading)
        };
        block(view, self.node)
    }
}

impl Render for HeadedNode<'_> {
    fn render(&self) -> String {
        block(BlockView::of(self.kind, self.heading), self.node)
    }
}

/// The block `view` of `node`, with its backgrounds, its steps and the blocks below it.
fn block(view: BlockView<'_>, node: SiteTreeNode<'_>) -> String {
    let page = node.feature_page().map(|feature| feature.page());
    let option = page.and_then(|page| page.backgrounds_option.as_ref());
    let backgrounds = backgrounds_in_scenario(node, &view, option.is_some_and(|o| o.inline));
    let steps = own_steps(node);
    let steps_heading = steps_heading(option, view.level.below(), &backgrounds, &steps);
    let runs = page.map_or("", |page| page.words.runs_for_each_row.as_str());
    rendered(&BlockTemplate {
        backgrounds,
        steps_heading,
        steps,
        examples: examples_list(node, runs),
        blocks: pieces(node.children().filter(is_block)).join("\n"),
        block: view,
    })
}

/// Whether `node` is a block with a heading of its own.
fn is_block(node: &SiteTreeNode<'_>) -> bool {
    !node.is_step() && !is_examples(node)
}

/// The steps directly below `node`, as one list; empty when it has none.
fn own_steps(node: SiteTreeNode<'_>) -> String {
    let steps = steps_of(node);
    if steps.is_empty() {
        return String::new();
    }
    step_list(steps)
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
            collapsed_when_unchecked: None,
            tags: &[],
            description: &None,
        }
    }
}

impl BlockKind {
    /// The class the stylesheet and the script know the block by.
    fn class(self) -> &'static str {
        match self {
            BlockKind::Background => "ghg-background",
            BlockKind::BackgroundInScenario => "ghg-background-in-scenario",
            BlockKind::Rule => "ghg-rule",
            BlockKind::OutsideRules => "ghg-outside-rules",
            BlockKind::Scenario => "ghg-scenario",
            BlockKind::ExamplesAsScenarios => "ghg-examples-as-scenarios",
        }
    }
}
