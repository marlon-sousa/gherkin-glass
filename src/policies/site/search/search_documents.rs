//! Policy: the documents search reads, one for each feature, rule and scenario, from the feature pages of the site tree and their blocks.

use crate::entities::{
    RuleBlock, ScenarioBlock, SearchDocument, SearchKind, SiteAddress, SiteFeaturePage, SiteNode,
    SiteTreeNode, TagView,
};

/// The documents of every page, in the order of `pages`, and within a page in file order.
pub(crate) fn search_documents(pages: &[SiteFeaturePage<'_>]) -> Vec<SearchDocument> {
    pages.iter().flat_map(page_documents).collect()
}

/// The documents of one page: its feature's, then each rule's and scenario's.
fn page_documents(feature: &SiteFeaturePage<'_>) -> Vec<SearchDocument> {
    let page = feature.page();
    let on_page = OnPage {
        feature: &page.name,
        address: &page.address,
    };
    let feature_document = SearchDocument {
        kind: SearchKind::Feature,
        name: page.name.clone(),
        feature: None,
        rule: None,
        steps: background_steps(feature.blocks()),
        tags: tags(&page.tags),
        page: page.address.clone(),
        anchor: String::new(),
    };
    let blocks = feature.blocks().flat_map(|block| match block.value() {
        SiteNode::Scenario(scenario) => vec![on_page.scenario(scenario, block, None)],
        SiteNode::Rule(rule) => on_page.rule(rule, block),
        _ => Vec::new(),
    });
    [feature_document].into_iter().chain(blocks).collect()
}

/// What every document of a page shares.
struct OnPage<'a> {
    feature: &'a str,
    address: &'a SiteAddress,
}

impl OnPage<'_> {
    /// The document of `rule`, whose node is `node`, then its scenarios'.
    fn rule(&self, rule: &RuleBlock, node: SiteTreeNode<'_>) -> Vec<SearchDocument> {
        let document = SearchDocument {
            kind: SearchKind::Rule,
            name: rule.name.clone(),
            feature: Some(self.feature.to_string()),
            rule: None,
            steps: background_steps(node.children()),
            tags: tags(&rule.tags),
            page: self.address.clone(),
            anchor: rule.heading.anchor.clone(),
        };
        let scenarios = node.children().filter_map(|child| match child.value() {
            SiteNode::Scenario(scenario) => Some(self.scenario(scenario, child, Some(&rule.name))),
            _ => None,
        });
        [document].into_iter().chain(scenarios).collect()
    }

    /// The document of `scenario`, whose node is `node`, in the rule named `rule`, if any.
    fn scenario(
        &self,
        scenario: &ScenarioBlock,
        node: SiteTreeNode<'_>,
        rule: Option<&str>,
    ) -> SearchDocument {
        let examples: Vec<_> = node
            .children()
            .filter_map(|child| match child.value() {
                SiteNode::Examples(examples) => Some(examples),
                _ => None,
            })
            .collect();
        SearchDocument {
            kind: if examples.is_empty() {
                SearchKind::Scenario
            } else {
                SearchKind::ScenarioOutline
            },
            name: scenario.name.clone(),
            feature: Some(self.feature.to_string()),
            rule: rule.map(str::to_string),
            steps: steps_of(node),
            tags: own_then_examples_tags(&scenario.tags, examples.iter().map(|e| &e.tags)),
            page: self.address.clone(),
            anchor: scenario.heading.anchor.clone(),
        }
    }
}

/// The steps of the background among `blocks`, the blocks of a page or a rule; none without one.
fn background_steps<'a>(blocks: impl Iterator<Item = SiteTreeNode<'a>>) -> Vec<String> {
    blocks
        .filter(|block| matches!(block.value(), SiteNode::Background(_)))
        .flat_map(steps_of)
        .collect()
}

/// The text of each step directly below `block`.
fn steps_of(block: SiteTreeNode<'_>) -> Vec<String> {
    block
        .children()
        .filter_map(|child| match child.value() {
            SiteNode::Step(step) => Some(step.text()),
            _ => None,
        })
        .collect()
}

fn tags(tags: &[TagView]) -> Vec<String> {
    tags.iter().map(|tag| tag.text.clone()).collect()
}

/// A scenario's own tags, then, for an outline, those of its examples blocks, each once.
fn own_then_examples_tags<'a>(
    own: &[TagView],
    examples: impl Iterator<Item = &'a Vec<TagView>>,
) -> Vec<String> {
    let mut found = tags(own);
    for tag in examples.flat_map(|tags| self::tags(tags)) {
        if !found.contains(&tag) {
            found.push(tag);
        }
    }
    found
}
