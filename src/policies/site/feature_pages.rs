//! Policy: a feature's page, its blocks, its anchors, its outlines' rows and its steps' highlighting.

mod anchor_rule;
mod example_rows;
mod page_building;
mod step_highlighting;

pub(crate) use anchor_rule::anchor_feature;
pub(crate) use anchor_rule::{anchor_of, unique};
pub(crate) use example_rows::{ExampleRow, example_rows};
pub(crate) use page_building::{build_page, heading};
pub(crate) use step_highlighting::highlight_step;
