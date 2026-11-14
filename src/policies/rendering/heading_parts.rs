//! Policy: the words of a heading, rendered from their template, the feature text in them isolated, and in its feature's language where the page is in another.

use askama::Template;

use super::node_rendering::{Render, rendered};
use crate::entities::HeadingPart;

#[derive(Template)]
#[template(path = "heading_parts.html")]
struct HeadingPartsTemplate<'a> {
    parts: &'a [HeadingPart],
    language: Option<&'a str>,
}

impl Render for [HeadingPart] {
    fn render(&self) -> String {
        self.render_in(None)
    }
}

/// A heading shown on a page in a language other than its feature's, such
/// as a folder page: only its feature text takes the feature's `lang`, so
/// that gherkin-glass's own words are read in the page's.
pub(super) trait RenderIn {
    fn render_in(&self, language: Option<&str>) -> String;
}

impl RenderIn for [HeadingPart] {
    fn render_in(&self, language: Option<&str>) -> String {
        rendered(&HeadingPartsTemplate {
            parts: self,
            language,
        })
    }
}
