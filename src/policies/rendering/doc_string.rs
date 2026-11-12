//! Policy: a step's doc string, captioned with its media type when it has one.

use askama::Template;

use super::node_rendering::{Render, rendered};
use crate::entities::DocStringView;

#[derive(Template)]
#[template(path = "doc_string.html")]
struct DocStringTemplate<'a> {
    doc: &'a DocStringView,
}

impl Render for DocStringView {
    fn render(&self) -> String {
        rendered(&DocStringTemplate { doc: self })
    }
}
