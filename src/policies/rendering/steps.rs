//! Policy: a list of steps and each step in it, its text in the parts its highlighting tells apart, with its table or its doc string, from the step nodes below a block.

use askama::Template;

use super::node_rendering::{NodeOf, Render, pieces, rendered};
use crate::entities::{SiteTreeNode, StepPart, StepView};

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
    pieces(block.children().filter(SiteTreeNode::is_step))
}

impl Render for NodeOf<'_, StepView> {
    fn render(&self) -> String {
        rendered(&StepTemplate {
            step: self.value,
            attachments: pieces(self.node.children()),
        })
    }
}
