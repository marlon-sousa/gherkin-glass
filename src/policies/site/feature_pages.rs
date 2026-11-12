//! Policy: a feature's page, its blocks, its anchors, its outlines' rows and its steps' highlighting.

mod anchor_rule;
mod example_rows;
mod page_building;
mod step_highlighting;

pub(super) use page_building::build_page;

use anchor_rule::anchor_feature;
use example_rows::{ExampleRow, example_rows};
use step_highlighting::highlight_step;
