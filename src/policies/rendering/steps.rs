//! Policy: a list of steps and each step in it, its text in the parts its highlighting tells apart, with its table or its doc string.

use askama::Template;

use super::node_rendering::{Render, rendered};
use crate::entities::{StepPart, StepView};

/// Steps as one list.
pub(super) struct StepList<'a> {
    pub(super) steps: &'a [StepView],
    /// The `id` of the words that introduce a background's steps in a scenario.
    pub(super) introduction: Option<&'a str>,
    pub(super) shown: bool,
}

#[derive(Template)]
#[template(path = "step.html")]
struct StepTemplate<'a> {
    step: &'a StepView,
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

/// A block's own steps.
impl Render for [StepView] {
    fn render(&self) -> String {
        StepList {
            steps: self,
            introduction: None,
            shown: true,
        }
        .render()
    }
}

impl Render for StepView {
    fn render(&self) -> String {
        rendered(&StepTemplate { step: self })
    }
}
