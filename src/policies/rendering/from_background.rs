//! Policy: a background whose steps a scenario starts with, as its introduction and the list of its steps.

use askama::Template;

use super::node_rendering::{Render, rendered};
use super::steps::StepList;
use crate::entities::FromBackground;

#[derive(Template)]
#[template(path = "from_background.html")]
struct FromBackgroundTemplate<'a> {
    from: &'a FromBackground,
    steps: String,
}

impl Render for FromBackground {
    fn render(&self) -> String {
        rendered(&FromBackgroundTemplate {
            from: self,
            steps: StepList {
                steps: &self.steps,
                introduction: Some(&self.id),
                shown: self.inline,
            }
            .render(),
        })
    }
}
