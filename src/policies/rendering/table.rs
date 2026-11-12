//! Policy: a table, a step's or an examples block's, with its header row.

use askama::Template;

use super::node_rendering::{Render, rendered};
use crate::entities::TableView;

#[derive(Template)]
#[template(path = "table.html")]
struct TableTemplate<'a> {
    table: &'a TableView,
}

impl Render for TableView {
    fn render(&self) -> String {
        rendered(&TableTemplate { table: self })
    }
}
