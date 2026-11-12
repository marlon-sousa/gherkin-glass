//! Entity: a feature page of the site tree, with its blocks below it, and what the folder pages and the folder tree read from them: its contents and how many rules and scenarios it has.

use ego_tree::NodeRef;

use super::{SiteNode, SiteTreeNode};
use crate::entities::{BlockHeading, ContentsEntry, Page};

/// A feature page node of the site tree.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SiteFeaturePage<'a> {
    page: &'a Page,
    node: NodeRef<'a, SiteNode>,
}

impl<'a> SiteFeaturePage<'a> {
    /// `node` as a feature page, when it is one.
    pub(super) fn of(node: NodeRef<'a, SiteNode>) -> Option<SiteFeaturePage<'a>> {
        match node.value() {
            SiteNode::FeaturePage(page) => Some(SiteFeaturePage { page, node }),
            _ => None,
        }
    }

    pub(crate) fn page(&self) -> &'a Page {
        self.page
    }

    /// The blocks directly in the page, in the order of the file: the
    /// feature's background, its scenarios, then its rules.
    pub(crate) fn blocks(&self) -> impl Iterator<Item = SiteTreeNode<'a>> + use<'a> {
        self.node.children().map(SiteTreeNode::new)
    }

    /// Every scenario of the page, those in its rules among them, in the order of the file.
    pub(crate) fn scenarios(&self) -> impl Iterator<Item = SiteTreeNode<'a>> + use<'a> {
        self.node
            .children()
            .flat_map(|block| [block].into_iter().chain(rule_children(block)))
            .filter(is_scenario)
            .map(SiteTreeNode::new)
    }

    /// The page's rules and scenarios, a rule's entry listing its scenarios.
    pub(crate) fn contents(&self) -> Vec<ContentsEntry> {
        self.node.children().filter_map(contents_entry).collect()
    }

    pub(crate) fn rule_count(&self) -> usize {
        self.node.children().filter(is_rule).count()
    }

    /// Scenario outlines among them, and those inside rules.
    pub(crate) fn scenario_count(&self) -> usize {
        self.node.children().map(scenarios_in).sum()
    }
}

fn contents_entry(node: NodeRef<'_, SiteNode>) -> Option<ContentsEntry> {
    match node.value() {
        SiteNode::Rule(rule) => Some(ContentsEntry {
            entries: node.children().filter_map(scenario_entry).collect(),
            ..entry(&rule.heading)
        }),
        _ => scenario_entry(node),
    }
}

fn scenario_entry(node: NodeRef<'_, SiteNode>) -> Option<ContentsEntry> {
    match node.value() {
        SiteNode::Scenario(scenario) => Some(entry(&scenario.heading)),
        _ => None,
    }
}

fn entry(heading: &BlockHeading) -> ContentsEntry {
    ContentsEntry {
        anchor: heading.anchor.clone(),
        heading: heading.text(),
        entries: Vec::new(),
    }
}

fn is_rule(node: &NodeRef<'_, SiteNode>) -> bool {
    matches!(node.value(), SiteNode::Rule(_))
}

/// The scenarios the block `node` is or holds.
fn scenarios_in(node: NodeRef<'_, SiteNode>) -> usize {
    match node.value() {
        SiteNode::Rule(_) => node.children().filter(is_scenario).count(),
        SiteNode::Scenario(_) => 1,
        _ => 0,
    }
}

/// The blocks in `block` when it is a rule; none for any other block.
fn rule_children(block: NodeRef<'_, SiteNode>) -> impl Iterator<Item = NodeRef<'_, SiteNode>> {
    is_rule(&block)
        .then(|| block.children())
        .into_iter()
        .flatten()
}

fn is_scenario(node: &NodeRef<'_, SiteNode>) -> bool {
    matches!(node.value(), SiteNode::Scenario(_))
}
