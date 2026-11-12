//! Entity: a file the input tree hands out to be parsed.

use std::path::PathBuf;

use ego_tree::NodeId;

/// Made only by the input tree, which keeps its node.
#[derive(Debug)]
pub(crate) struct FileToParse {
    pub(super) node: NodeId,
    pub(crate) path: PathBuf,
}
