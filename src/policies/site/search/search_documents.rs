//! Policy: the documents search reads, one for each feature, rule, background, scenario and examples block, from the feature pages of the site tree and their blocks.

use crate::entities::{
    BackgroundBlock, ExamplesBlock, OutlinePlace, RuleBlock, ScenarioBlock, SearchDocument,
    SearchKind, SiteAddress, SiteFeaturePage, SiteNode, SiteTreeNode, StepView, TableView, TagView,
};

/// The documents of every page, in the order of `pages`, and within a page in file order.
pub(crate) fn search_documents(pages: &[SiteFeaturePage<'_>]) -> Vec<SearchDocument> {
    pages.iter().flat_map(page_documents).collect()
}

/// The documents of one page: its feature's, then each of its blocks' in file order.
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
        outline: None,
        steps: Vec::new(),
        tags: tags(&page.tags),
        cells: Vec::new(),
        page: page.address.clone(),
        anchor: String::new(),
    };
    let blocks = feature
        .blocks()
        .flat_map(|block| on_page.block(block, None));
    [feature_document].into_iter().chain(blocks).collect()
}

/// What every document of a page shares.
struct OnPage<'a> {
    feature: &'a str,
    address: &'a SiteAddress,
}

impl OnPage<'_> {
    /// The documents of `node`, a block directly in the page or in the rule named `rule`: a
    /// background's, a scenario's then its examples blocks', those of the scenarios outside
    /// every rule in their block, or a rule's then its blocks'.
    fn block(&self, node: SiteTreeNode<'_>, rule: Option<&str>) -> Vec<SearchDocument> {
        match node.value() {
            SiteNode::Background(background) => vec![self.background(background, node, rule)],
            SiteNode::Scenario(scenario) => self.scenario(scenario, node, rule),
            SiteNode::OutsideRules(_) => node
                .children()
                .flat_map(|child| self.block(child, None))
                .collect(),
            SiteNode::Rule(rule) => self.rule(rule, node),
            _ => Vec::new(),
        }
    }

    /// The document of `rule`, whose node is `node`, then its blocks'.
    fn rule(&self, rule: &RuleBlock, node: SiteTreeNode<'_>) -> Vec<SearchDocument> {
        let document = self.document(SearchKind::Rule, &rule.name, None, &rule.heading.anchor);
        let document = SearchDocument {
            tags: tags(&rule.tags),
            ..document
        };
        let blocks = node
            .children()
            .flat_map(|child| self.block(child, Some(&rule.name)));
        [document].into_iter().chain(blocks).collect()
    }

    /// The document of `background`, whose node is `node`, in the rule named `rule`, if any.
    fn background(
        &self,
        background: &BackgroundBlock,
        node: SiteTreeNode<'_>,
        rule: Option<&str>,
    ) -> SearchDocument {
        SearchDocument {
            steps: steps_of(node),
            ..self.document(
                SearchKind::Background,
                &background.name,
                rule,
                &background.heading.anchor,
            )
        }
    }

    /// The document of `scenario`, whose node is `node`, in the rule named `rule`, if any, then
    /// those of its examples blocks.
    fn scenario(
        &self,
        scenario: &ScenarioBlock,
        node: SiteTreeNode<'_>,
        rule: Option<&str>,
    ) -> Vec<SearchDocument> {
        let examples: Vec<_> = node.examples().collect();
        let kind = if examples.is_empty() {
            SearchKind::Scenario
        } else {
            SearchKind::ScenarioOutline
        };
        let document = SearchDocument {
            steps: steps_of(node),
            tags: own_then_examples_tags(
                &scenario.tags,
                examples.iter().copied().map(examples_tags),
            ),
            ..self.document(kind, &scenario.name, rule, &scenario.heading.anchor)
        };
        let examples = examples
            .into_iter()
            .zip(1..)
            .map(|(block, place)| self.examples(block, rule, &scenario.name, place));
        [document].into_iter().chain(examples).collect()
    }

    /// The document of `examples`, at `place` among the blocks of the outline named `outline`,
    /// in the rule named `rule`, if any.
    fn examples(
        &self,
        examples: &ExamplesBlock,
        rule: Option<&str>,
        outline: &str,
        place: usize,
    ) -> SearchDocument {
        SearchDocument {
            outline: Some(OutlinePlace {
                name: outline.to_string(),
                place,
            }),
            tags: tags(&examples.tags),
            cells: examples.table.iter().flat_map(cells).collect(),
            ..self.document(SearchKind::Examples, &examples.name, rule, &examples.anchor)
        }
    }

    /// A document of `kind` on this page, named `name`, in the rule named `rule`, if any, at
    /// `anchor`, with no outline, step, tag or cell.
    fn document(
        &self,
        kind: SearchKind,
        name: &str,
        rule: Option<&str>,
        anchor: &str,
    ) -> SearchDocument {
        SearchDocument {
            kind,
            name: name.to_string(),
            feature: Some(self.feature.to_string()),
            rule: rule.map(str::to_string),
            outline: None,
            steps: Vec::new(),
            tags: Vec::new(),
            cells: Vec::new(),
            page: self.address.clone(),
            anchor: anchor.to_string(),
        }
    }
}

/// The text of each step directly below `block`.
fn steps_of(block: SiteTreeNode<'_>) -> Vec<String> {
    block.steps().map(StepView::text).collect()
}

/// Each cell of `table`, row by row from its header.
fn cells(table: &TableView) -> Vec<String> {
    [&table.header]
        .into_iter()
        .chain(&table.rows)
        .flatten()
        .cloned()
        .collect()
}

fn examples_tags(examples: &ExamplesBlock) -> &Vec<TagView> {
    &examples.tags
}

fn tags(tags: &[TagView]) -> Vec<String> {
    tags.iter().map(tag_text).collect()
}

fn tag_text(tag: &TagView) -> String {
    tag.text.clone()
}

/// A scenario's own tags, then, for an outline, those of its examples blocks, each once.
fn own_then_examples_tags<'a>(
    own: &[TagView],
    examples: impl Iterator<Item = &'a Vec<TagView>>,
) -> Vec<String> {
    let mut found = tags(own);
    for tag in examples.flat_map(Vec::as_slice).map(tag_text) {
        if !found.contains(&tag) {
            found.push(tag);
        }
    }
    found
}
