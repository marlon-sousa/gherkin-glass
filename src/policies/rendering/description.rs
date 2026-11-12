//! Policy: the description of a feature or a block, one paragraph with its lines kept apart.

use askama::Template;

use super::node_rendering::{Render, rendered};
use crate::entities::Paragraph;

#[derive(Template)]
#[template(path = "description.html")]
struct DescriptionTemplate<'a> {
    paragraph: &'a Paragraph,
}

impl Render for Paragraph {
    fn render(&self) -> String {
        rendered(&DescriptionTemplate { paragraph: self })
    }
}
