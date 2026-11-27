//! Entity: what a build made, which the next build under `--watch` starts from.

use std::sync::Arc;

use crate::entities::{InputTree, Settings};

#[derive(Debug, Clone)]
pub(crate) struct BuildSnapshot {
    /// Shared with every snapshot made from this one while the settings read again are equal.
    pub(crate) settings: Arc<Settings>,
    /// Holds every entry of the input as the build last found it, also when the input stopped
    /// the build.
    pub(crate) input_tree: InputTree,
}
