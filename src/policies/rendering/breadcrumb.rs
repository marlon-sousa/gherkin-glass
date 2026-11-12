//! Policy: a page's breadcrumb, rendered from its template.

use askama::Template;

use super::node_rendering::{Render, rendered};
use crate::entities::Breadcrumb;

#[derive(Template)]
#[template(path = "breadcrumb.html")]
struct BreadcrumbTemplate<'a> {
    breadcrumb: &'a Breadcrumb,
}

impl Render for Breadcrumb {
    fn render(&self) -> String {
        rendered(&BreadcrumbTemplate { breadcrumb: self })
    }
}
