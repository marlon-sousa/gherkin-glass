//! Policy: where a tag is written on a feature's page, and which of its scenarios carry it, themselves or through the blocks they are in.

use crate::entities::{ScenarioBlock, SiteFeaturePage, SiteNode, SiteTreeNode, TagView};

/// How a scenario carries a tag: the first of these that has it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Through {
    Itself,
    Examples,
    Rule,
    Feature,
}

/// The scenarios of `page` that carry `tag`, in the order of the file.
pub(super) fn carriers<'a>(
    page: &SiteFeaturePage<'a>,
    tag: &str,
) -> Vec<(&'a ScenarioBlock, Through)> {
    page.scenarios()
        .filter_map(|scenario| carrier(scenario, tag))
        .collect()
}

/// The scenario `node`, with how it carries `tag`: itself, in its examples,
/// or found among the blocks it is in, the nearest first.
fn carrier<'a>(node: SiteTreeNode<'a>, tag: &str) -> Option<(&'a ScenarioBlock, Through)> {
    let SiteNode::Scenario(scenario) = node.value() else {
        return None;
    };
    let through = if has(&scenario.tags, tag) {
        Some(Through::Itself)
    } else if in_examples(node, tag) {
        Some(Through::Examples)
    } else {
        node.enclosing_blocks()
            .find_map(|block| inherited(block.value(), tag))
    };
    through.map(|through| (scenario, through))
}

/// How a scenario inherits `tag` from `block`, a rule or a feature page, when the block has it.
fn inherited(block: &SiteNode, tag: &str) -> Option<Through> {
    match block {
        SiteNode::Rule(rule) => has(&rule.tags, tag).then_some(Through::Rule),
        SiteNode::FeaturePage(page) => has(&page.tags, tag).then_some(Through::Feature),
        _ => None,
    }
}

/// Whether `tag` is written anywhere on `page`.
pub(super) fn writes(page: &SiteFeaturePage<'_>, tag: &str) -> bool {
    has(&page.page().tags, tag)
        || page.blocks().any(|block| match block.value() {
            SiteNode::Rule(rule) => has(&rule.tags, tag),
            _ => false,
        })
        || page
            .scenarios()
            .any(|scenario| scenario_writes(scenario, tag))
}

fn scenario_writes(node: SiteTreeNode<'_>, tag: &str) -> bool {
    matches!(node.value(), SiteNode::Scenario(scenario) if has(&scenario.tags, tag))
        || in_examples(node, tag)
}

/// Whether one of the outline `node`'s examples blocks has `tag`.
fn in_examples(node: SiteTreeNode<'_>, tag: &str) -> bool {
    node.children().any(|child| match child.value() {
        SiteNode::Examples(examples) => has(&examples.tags, tag),
        _ => false,
    })
}

fn has(tags: &[TagView], tag: &str) -> bool {
    tags.iter().any(|view| view.text == tag)
}
