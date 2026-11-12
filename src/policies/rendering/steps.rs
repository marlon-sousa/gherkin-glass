//! Policy: a list of steps and each step in it, its text in the parts its highlighting tells apart, with its table or its doc string, from the step nodes below a block.

use askama::Template;

use super::node_rendering::{Render, rendered};
use crate::entities::{SiteNode, SiteTreeNode, StepPart, StepView};

/// Steps as one list, each already rendered.
pub(super) struct StepList<'a> {
    pub(super) steps: Vec<String>,
    /// The `id` of the words that introduce a background's steps in a scenario.
    pub(super) introduction: Option<&'a str>,
    pub(super) shown: bool,
}

#[derive(Template)]
#[template(path = "step.html")]
struct StepTemplate<'a> {
    step: &'a StepView,
    /// Its data table or its doc string, already rendered.
    attachments: Vec<String>,
}

#[derive(Template)]
#[template(path = "steps.html")]
struct StepsTemplate<'a> {
    list: &'a StepList<'a>,
}

impl Render for StepList<'_> {
    fn render(&self) -> String {
        rendered(&StepsTemplate { list: self })
    }
}

/// Each step node directly below `block`, rendered, in order.
pub(super) fn steps_of(block: SiteTreeNode<'_>) -> Vec<String> {
    block.children().filter_map(step).collect()
}

/// The step of `node`, when it is one, with what is below it.
fn step(node: SiteTreeNode<'_>) -> Option<String> {
    let SiteNode::Step(step) = node.value() else {
        return None;
    };
    Some(rendered(&StepTemplate {
        step,
        attachments: node.children().filter_map(attachment).collect(),
    }))
}

/// A step's data table or doc string, rendered from its node.
fn attachment(node: SiteTreeNode<'_>) -> Option<String> {
    match node.value() {
        SiteNode::StepTable(table) => Some(table.render()),
        SiteNode::DocString(doc_string) => Some(doc_string.render()),
        _ => None,
    }
}
