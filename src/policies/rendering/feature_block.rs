//! Policy: the blocks of a feature's page, each kind of block node rendered through the one template of a block, with the backgrounds a scenario starts with, its steps and the blocks below it.

use askama::Template;

use super::from_background::FromBackground;
use super::node_rendering::{Render, rendered};
use super::steps::{StepList, steps_of};
use crate::entities::{
    BlockHeading, HeadingLevel, HeadingPart, Paragraph, SiteNode, SiteTreeNode, TableView, TagView,
};

#[derive(Template)]
#[template(path = "feature_block.html")]
struct BlockTemplate<'a> {
    block: BlockView<'a>,
    /// The backgrounds a scenario starts with, already rendered.
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
    tags: &'a [TagView],
    description: &'a Option<Paragraph>,
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

/// The blocks among `nodes`, one after the other, a line apart, each with
/// what is below it; the nodes that are not blocks, such as steps, left out.
pub(super) fn blocks<'a>(nodes: impl Iterator<Item = SiteTreeNode<'a>>) -> String {
    nodes.filter_map(block).collect::<Vec<_>>().join("\n")
}

/// The block of `node`, when it is one.
fn block(node: SiteTreeNode<'_>) -> Option<String> {
    let view = match node.value() {
        SiteNode::Background(background) => BlockView {
            description: &background.description,
            ..BlockView::of(BlockKind::Background, &background.heading)
        },
        SiteNode::Rule(rule) => BlockView {
            tags: &rule.tags,
            description: &rule.description,
            ..BlockView::of(BlockKind::Rule, &rule.heading)
        },
        SiteNode::Scenario(scenario) => BlockView {
            tags: &scenario.tags,
            description: &scenario.description,
            ..BlockView::of(BlockKind::Scenario, &scenario.heading)
        },
        SiteNode::Examples(examples) => BlockView {
            tags: &examples.tags,
            description: &examples.description,
            table: &examples.table,
            ..BlockView::of(BlockKind::Examples, &examples.heading)
        },
        SiteNode::ExamplesAsScenarios(heading) => {
            BlockView::of(BlockKind::ExamplesAsScenarios, heading)
        }
        SiteNode::ExampleScenario(heading) => BlockView::of(BlockKind::Scenario, heading),
        _ => return None,
    };
    Some(rendered(&BlockTemplate {
        backgrounds: from_backgrounds(node, view.anchor),
        steps: own_steps(node),
        blocks: blocks(node.children()),
        block: view,
    }))
}

/// The backgrounds the scenario `node`, at `anchor`, starts with; none for
/// any other block.
fn from_backgrounds(node: SiteTreeNode<'_>, anchor: &str) -> Vec<String> {
    let inline = node
        .feature_page()
        .and_then(|feature| feature.page().backgrounds_option.as_ref())
        .is_some_and(|option| option.inline);
    node.backgrounds()
        .into_iter()
        .zip(1..)
        .map(|((background, background_node), place)| {
            FromBackground {
                background,
                node: background_node,
                scenario: anchor,
                place,
                inline,
            }
            .render()
        })
        .collect()
}

/// The steps directly below `node`, as one list; empty when it has none.
fn own_steps(node: SiteTreeNode<'_>) -> String {
    let steps = steps_of(node);
    if steps.is_empty() {
        return String::new();
    }
    StepList {
        steps,
        introduction: None,
        shown: true,
    }
    .render()
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
