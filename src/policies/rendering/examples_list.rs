//! Policy: an outline's list of examples, after its steps, each examples block an item of it, with its runs, rendered through the rendering trait from its template.

use askama::Template;

use super::node_rendering::{NodeOf, Render, pieces, rendered};
use crate::entities::{ExamplesBlock, SiteNode, SiteTreeNode};

#[derive(Template)]
#[template(path = "examples_list.html")]
struct ExamplesListTemplate<'a> {
    /// The words before the list, with their `id`; `None` when no block has
    /// a row to run.
    words: Option<(&'a str, &'a str)>,
    /// Its items, already rendered.
    items: Vec<String>,
}

#[derive(Template)]
#[template(path = "examples_item.html")]
struct ExamplesItemTemplate<'a> {
    examples: &'a ExamplesBlock,
    /// The block of its runs, already rendered; empty when it has no row.
    runs: String,
}

impl Render for NodeOf<'_, ExamplesBlock> {
    fn render(&self) -> String {
        rendered(&ExamplesItemTemplate {
            examples: self.value,
            runs: pieces(self.node.children()).join("\n"),
        })
    }
}

/// The list of the examples blocks directly below `outline`, after `words`
/// when one of them has a row; empty for a block with none.
pub(super) fn examples_list(outline: SiteTreeNode<'_>, words: &str) -> String {
    let items = pieces(outline.children().filter(is_examples));
    if items.is_empty() {
        return String::new();
    }
    let runs_anchor = match outline.value() {
        SiteNode::Scenario(scenario) => scenario.runs_anchor.as_deref(),
        _ => None,
    };
    rendered(&ExamplesListTemplate {
        words: runs_anchor.map(|anchor| (words, anchor)),
        items,
    })
}

pub(super) fn is_examples(node: &SiteTreeNode<'_>) -> bool {
    matches!(node.value(), SiteNode::Examples(_))
}
