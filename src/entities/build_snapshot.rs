//! Entity: what a build made, which the next build under `--watch` is judged against.

use crate::entities::{InputTree, Settings};

#[derive(Debug, Clone)]
pub(crate) struct BuildSnapshot {
    pub(crate) settings: Settings,
    pub(crate) input_tree: InputTree,
}
