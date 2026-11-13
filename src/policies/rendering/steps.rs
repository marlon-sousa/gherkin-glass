//! Policy: a list of steps and each step in it, its text in the parts its highlighting tells apart, with its table or its doc string, from the step nodes below a block.

use askama::Template;

use super::node_rendering::{NodeOf, Render, pieces, rendered};
use crate::entities::{SiteTreeNode, StepPart, StepView};

#[derive(Template)]
#[template(path = "step.html")]
struct StepTemplate<'a> {
    step: &'a StepView,
    /// Its data table or its doc string, already rendered.
    attachments: Vec<String>,
}

/// Steps as one list, each already rendered.
#[derive(Template)]
#[template(path = "steps.html")]
struct StepsTemplate {
    steps: Vec<String>,
}

/// `steps`, each already rendered, as one numbered list.
pub(super) fn step_list(steps: Vec<String>) -> String {
    rendered(&StepsTemplate { steps })
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
