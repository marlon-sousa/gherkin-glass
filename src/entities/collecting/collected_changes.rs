//! Entity: what the input collector makes of the paths that changed since the last build: the new input tree, and whether they call for a rebuild.

use crate::entities::InputTree;

#[derive(Debug, Clone)]
pub(crate) struct CollectedChanges {
    /// Made from the last build's tree and the paths, kept whether they call for a rebuild or not.
    pub(crate) input_tree: InputTree,
    pub(crate) calls_for_rebuild: bool,
}
