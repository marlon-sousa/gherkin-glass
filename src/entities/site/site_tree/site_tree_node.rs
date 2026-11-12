//! Entity: a node of the site tree as the stages that read it see it: its value, the nodes below it, and what it finds among the nodes above it.

use ego_tree::NodeRef;

use super::{SiteFeaturePage, SiteFolderPage, SiteNode};
use crate::entities::{BackgroundBlock, ExamplesBlock, StepView};

/// A node of the site tree, with every node above and below it.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SiteTreeNode<'a> {
    node: NodeRef<'a, SiteNode>,
}

impl<'a> SiteTreeNode<'a> {
    pub(super) fn new(node: NodeRef<'a, SiteNode>) -> SiteTreeNode<'a> {
        SiteTreeNode { node }
    }

    pub(crate) fn value(&self) -> &'a SiteNode {
        self.node.value()
    }

    /// The nodes directly below this one, in the order they were put in.
    pub(crate) fn children(&self) -> impl Iterator<Item = SiteTreeNode<'a>> + use<'a> {
        self.node.children().map(SiteTreeNode::new)
    }

    /// The node this one is directly below; `None` for the site.
    pub(crate) fn parent(&self) -> Option<SiteTreeNode<'a>> {
        self.node.parent().map(SiteTreeNode::new)
    }

    /// The steps directly below this node, in the order they were put in.
    pub(crate) fn steps(&self) -> impl Iterator<Item = &'a StepView> + use<'a> {
        self.node.children().filter_map(step_of)
    }

    /// The examples blocks directly below this node, an outline's, in the order they were put in.
    pub(crate) fn examples(&self) -> impl Iterator<Item = &'a ExamplesBlock> + use<'a> {
        self.node.children().filter_map(examples_of)
    }

    pub(crate) fn is_step(&self) -> bool {
        matches!(self.value(), SiteNode::Step(_))
    }

    /// The names of the folders of the input this node is in, from the outermost.
    pub(crate) fn folders(&self) -> Vec<&'a str> {
        let mut names: Vec<_> = self.node.ancestors().filter_map(folder_name).collect();
        names.reverse();
        names
    }

    /// The feature page this node is, or is part of.
    pub(crate) fn feature_page(&self) -> Option<SiteFeaturePage<'a>> {
        self.self_and_ancestors().find_map(SiteFeaturePage::of)
    }

    /// The folder page this node is, or is part of.
    pub(crate) fn folder_page(&self) -> Option<SiteFolderPage<'a>> {
        self.self_and_ancestors().find_map(SiteFolderPage::of)
    }

    /// The backgrounds whose steps this node, a scenario, starts with, each a
    /// background node beside it or beside a block it is in, with its steps
    /// below it: its feature's, then its rule's, those without steps left
    /// out. Empty for any other node.
    pub(crate) fn backgrounds(&self) -> Vec<(&'a BackgroundBlock, SiteTreeNode<'a>)> {
        if !matches!(self.value(), SiteNode::Scenario(_)) {
            return Vec::new();
        }
        let mut blocks: Vec<_> = self.enclosing_blocks().collect();
        blocks.reverse();
        blocks
            .into_iter()
            .flat_map(|block| block.children())
            .filter_map(background_with_steps)
            .collect()
    }

    /// The blocks this node is in, from the nearest: for a scenario, its rule,
    /// if it is in one, then its feature page.
    pub(crate) fn enclosing_blocks(&self) -> impl Iterator<Item = SiteTreeNode<'a>> + use<'a> {
        self.node
            .ancestors()
            .take_while(holds_blocks)
            .map(SiteTreeNode::new)
    }

    fn self_and_ancestors(&self) -> impl Iterator<Item = NodeRef<'a, SiteNode>> + use<'a> {
        [self.node].into_iter().chain(self.node.ancestors())
    }
}

/// Whether `node` is a feature page or a rule, the blocks a scenario can be in.
fn holds_blocks(node: &NodeRef<'_, SiteNode>) -> bool {
    matches!(node.value(), SiteNode::FeaturePage(_) | SiteNode::Rule(_))
}

fn step_of(node: NodeRef<'_, SiteNode>) -> Option<&StepView> {
    match node.value() {
        SiteNode::Step(step) => Some(step),
        _ => None,
    }
}

fn examples_of(node: NodeRef<'_, SiteNode>) -> Option<&ExamplesBlock> {
    match node.value() {
        SiteNode::Examples(examples) => Some(examples),
        _ => None,
    }
}

fn folder_name(node: NodeRef<'_, SiteNode>) -> Option<&str> {
    match node.value() {
        SiteNode::Folder(name) => Some(name),
        _ => None,
    }
}

/// `node`'s background, with the node, when it is a background with steps.
fn background_with_steps(node: SiteTreeNode<'_>) -> Option<(&BackgroundBlock, SiteTreeNode<'_>)> {
    let SiteNode::Background(background) = node.value() else {
        return None;
    };
    node.steps().next().is_some().then_some((background, node))
}

#[cfg(test)]
mod tests;
