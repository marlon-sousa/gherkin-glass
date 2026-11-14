//! Policy: an outline's list of examples, after its steps, each examples block an item of it rendered through the rendering trait from its template.

use askama::Template;

use super::node_rendering::{NodeOf, Render, pieces, rendered};
use crate::entities::{ExamplesBlock, SiteNode, SiteTreeNode};

#[derive(Template)]
#[template(path = "examples_list.html")]
struct ExamplesListTemplate<'a> {
    /// The words before the list.
    words: &'a str,
    /// Its items, already rendered.
    items: Vec<String>,
}

#[derive(Template)]
#[template(path = "examples_item.html")]
struct ExamplesItemTemplate<'a> {
    examples: &'a ExamplesBlock,
}

impl Render for NodeOf<'_, ExamplesBlock> {
    fn render(&self) -> String {
        rendered(&ExamplesItemTemplate {
            examples: self.value,
        })
    }
}

/// The list of the examples blocks directly below `outline`, after `words`;
/// empty for a block with none.
pub(super) fn examples_list(outline: SiteTreeNode<'_>, words: &str) -> String {
    let items = pieces(outline.children().filter(is_examples));
    if items.is_empty() {
        return String::new();
    }
    rendered(&ExamplesListTemplate { words, items })
}

pub(super) fn is_examples(node: &SiteTreeNode<'_>) -> bool {
    matches!(node.value(), SiteNode::Examples(_))
}
