//! Policy: a feature's page, its blocks, its anchors, its outlines' rows and its steps' highlighting.

mod anchor_rule;
mod example_rows;
mod page_building;
mod step_highlighting;

pub(super) use anchor_rule::{anchor_feature, anchor_of, unique};
pub(super) use page_building::build_page;

use example_rows::{ExampleRow, example_rows};
use step_highlighting::highlight_step;
