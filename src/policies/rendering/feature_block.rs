//! Policy: the blocks of a feature's page, each kind of block node rendered through the rendering trait into the one template of a block, with the backgrounds a scenario starts with, each a block of its own, its steps and the blocks below it.

use askama::Template;

use super::node_rendering::{NodeOf, Render, pieces, rendered};
use super::steps::{step_list, steps_of};
use crate::entities::{
    BackgroundBlock, BlockHeading, ExamplesBlock, HeadingLevel, HeadingPart, Paragraph, RuleBlock,
    ScenarioBlock, SiteTreeNode, TableView, TagView,
};

#[derive(Template)]
#[template(path = "feature_block.html")]
struct BlockTemplate<'a> {
    block: BlockView<'a>,
    /// The blocks of the backgrounds a scenario starts with, already rendered.
    backgrounds: Vec<String>,
    /// The block's own steps, already rendered; empty for a block with none.
    steps: String,
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
    table: &'a Option<TableView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockKind {
    Background,
    /// A background inside a scenario that starts with its steps.
    BackgroundInScenario,
    Rule,
    Scenario,
    Examples,
    ExamplesAsScenarios,
}

/// An outline's examples as scenarios, by its heading, with its node, which
/// holds the scenario of each row.
pub(super) struct ExamplesAsScenariosNode<'a> {
    pub(super) heading: &'a BlockHeading,
    pub(super) node: SiteTreeNode<'a>,
}

/// The scenario one row of an outline's examples makes, by its heading, with
/// its node, which holds its steps.
pub(super) struct ExampleScenarioNode<'a> {
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

impl Render for NodeOf<'_, ExamplesBlock> {
    fn render(&self) -> String {
        let view = BlockView {
            tags: &self.value.tags,
            description: &self.value.description,
            table: &self.value.table,
            ..BlockView::of(BlockKind::Examples, &self.value.heading)
        };
        block(view, self.node)
    }
}

impl Render for ExamplesAsScenariosNode<'_> {
    fn render(&self) -> String {
        block(
            BlockView::of(BlockKind::ExamplesAsScenarios, self.heading),
            self.node,
        )
    }
}

impl Render for ExampleScenarioNode<'_> {
    fn render(&self) -> String {
        block(BlockView::of(BlockKind::Scenario, self.heading), self.node)
    }
}

/// The block `view` of `node`, with its backgrounds, its steps and the blocks below it.
fn block(view: BlockView<'_>, node: SiteTreeNode<'_>) -> String {
    rendered(&BlockTemplate {
        backgrounds: backgrounds_in_scenario(node, &view),
        steps: own_steps(node),
        blocks: pieces(node.children().filter(is_block)).join("\n"),
        block: view,
    })
}

fn is_block(node: &SiteTreeNode<'_>) -> bool {
    !node.is_step()
}

/// The blocks of the backgrounds the scenario `node`, headed by `scenario`,
/// starts with, each one level below it; none for any other block.
fn backgrounds_in_scenario(node: SiteTreeNode<'_>, scenario: &BlockView<'_>) -> Vec<String> {
    let checked = node
        .feature_page()
        .and_then(|feature| feature.page().backgrounds_option.as_ref())
        .is_some_and(|option| option.inline);
    node.backgrounds()
        .into_iter()
        .zip(1..)
        .map(|((background, background_node), place)| {
            background_in_scenario(background, background_node, scenario, place, checked)
        })
        .collect()
}

/// The block of `background`, the `place`th the scenario headed by `scenario`
/// starts with, counted from 1, holding the steps below `background_node`;
/// it is expanded while the page's checkbox is `checked`.
fn background_in_scenario(
    background: &BackgroundBlock,
    background_node: SiteTreeNode<'_>,
    scenario: &BlockView<'_>,
    place: usize,
    checked: bool,
) -> String {
    let heading = BlockHeading {
        level: scenario.level.below(),
        anchor: format!("{}_background_{place}", scenario.anchor),
        parts: vec![HeadingPart {
            text: background.heading_in_scenario.clone(),
            isolated: false,
        }],
        collapsed: !checked,
    };
    let view = BlockView {
        collapsed_when_unchecked: Some(true),
        ..BlockView::of(BlockKind::BackgroundInScenario, &heading)
    };
    block(view, background_node)
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
            table: &None,
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
            BlockKind::Scenario => "ghg-scenario",
            BlockKind::Examples => "ghg-examples",
            BlockKind::ExamplesAsScenarios => "ghg-examples-as-scenarios",
        }
    }
}
