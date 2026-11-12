//! Policy: a background whose steps a scenario starts with, as its introduction and the list of its steps, from the background node the scenario finds among the nodes above it.

use askama::Template;

use super::node_rendering::{Render, rendered};
use super::steps::{StepList, steps_of};
use crate::entities::{BackgroundBlock, SiteTreeNode};

/// A background as a scenario shows it: inline, as a list of its steps, or
/// linked, as a link to the background.
pub(super) struct FromBackground<'a> {
    pub(super) background: &'a BackgroundBlock,
    /// The background's node, with its steps below it.
    pub(super) node: SiteTreeNode<'a>,
    /// The anchor of the scenario that starts with its steps.
    pub(super) scenario: &'a str,
    /// Its place among the scenario's backgrounds, from 1.
    pub(super) place: usize,
    /// Whether the page is written showing the steps rather than the link.
    pub(super) inline: bool,
}

#[derive(Template)]
#[template(path = "from_background.html")]
struct FromBackgroundTemplate<'a> {
    /// The `id` of the introduction, which names the list of steps.
    id: &'a str,
    introduction: &'a str,
    anchor: &'a str,
    heading: String,
    inline: bool,
    steps: String,
}

impl Render for FromBackground<'_> {
    fn render(&self) -> String {
        let id = format!("{}_background_{}", self.scenario, self.place);
        rendered(&FromBackgroundTemplate {
            id: &id,
            introduction: &self.background.introduction,
            anchor: &self.background.heading.anchor,
            heading: self.background.heading.text(),
            inline: self.inline,
            steps: StepList {
                steps: steps_of(self.node),
                introduction: Some(&id),
                shown: self.inline,
            }
            .render(),
        })
    }
}
