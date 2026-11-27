//! Policy: the rendering trait, through which every piece of a page renders itself beside its template, and what renders a template and a whole page.

use askama::Template;

/// A piece of a page, rendered. A piece has no line break before or after it: the template
/// that places it puts them.
pub(super) trait Render {
    fn render(&self) -> String;
}

/// The templates only write strings, which cannot fail.
#[expect(
    clippy::expect_used,
    reason = "a template that writes only strings renders"
)]
pub(super) fn rendered(template: &impl Template) -> String {
    template
        .render()
        .expect("a template that writes only strings renders")
}

/// A whole page, which ends in a line break.
pub(super) fn page(template: &impl Template) -> String {
    let mut html = rendered(template);
    if !html.ends_with('\n') {
        html.push('\n');
    }
    html
}
