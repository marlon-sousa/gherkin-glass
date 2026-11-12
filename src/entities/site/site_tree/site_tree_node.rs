//! Entity: a node of the site tree as the stages that read it see it: its value, the nodes below it, and what it finds among the nodes above it.

use ego_tree::NodeRef;

use super::{SiteFeaturePage, SiteNode};
use crate::entities::BackgroundBlock;

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

    /// The feature page this node is, or is part of.
    pub(crate) fn feature_page(&self) -> Option<SiteFeaturePage<'a>> {
        [self.node]
            .into_iter()
            .chain(self.node.ancestors())
            .find_map(SiteFeaturePage::of)
    }

    /// For a scenario, the backgrounds whose steps it starts with, each a
    /// background node beside it or beside a block it is in, with its steps
    /// below it: its feature's, then its rule's, those without steps left
    /// out. None for any other node.
    pub(crate) fn backgrounds(&self) -> Vec<(&'a BackgroundBlock, SiteTreeNode<'a>)> {
        if !matches!(self.value(), SiteNode::Scenario(_)) {
            return Vec::new();
        }
        let mut blocks: Vec<_> = self.enclosing_blocks().collect();
        blocks.reverse();
        blocks
            .into_iter()
            .flat_map(|block| block.node.children())
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
}

/// Whether `node` is a feature page or a rule, the blocks a scenario can be in.
fn holds_blocks(node: &NodeRef<'_, SiteNode>) -> bool {
    matches!(node.value(), SiteNode::FeaturePage(_) | SiteNode::Rule(_))
}

/// `node`'s background, with the node, when it is a background with steps.
fn background_with_steps(
    node: NodeRef<'_, SiteNode>,
) -> Option<(&BackgroundBlock, SiteTreeNode<'_>)> {
    let SiteNode::Background(background) = node.value() else {
        return None;
    };
    node.children()
        .any(|child| matches!(child.value(), SiteNode::Step(_)))
        .then_some((background, SiteTreeNode::new(node)))
}
