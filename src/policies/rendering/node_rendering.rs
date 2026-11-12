//! Policy: the rendering trait, through which every node of the site tree renders its own piece of a page, and what renders a file.

use askama::Template;

use crate::entities::SiteAddress;

/// A node of the site tree, rendered into its piece of a page. A piece has no
/// line break before or after it: the template that places it puts them.
pub(super) trait Render {
    fn render(&self) -> String;
}

/// A node that is a file of its own, which also says where the file goes.
pub(super) trait RenderFile: Render {
    fn address(&self) -> SiteAddress;
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
