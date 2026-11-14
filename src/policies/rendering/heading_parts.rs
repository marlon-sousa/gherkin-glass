//! Policy: the words of a heading on a feature's page, rendered from their template, the feature text in them isolated.

use askama::Template;

use super::node_rendering::{Render, rendered};
use crate::entities::HeadingPart;

#[derive(Template)]
#[template(path = "heading_parts.html")]
struct HeadingPartsTemplate<'a> {
    parts: &'a [HeadingPart],
}

impl Render for [HeadingPart] {
    fn render(&self) -> String {
        rendered(&HeadingPartsTemplate { parts: self })
    }
}
