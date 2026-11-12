//! Entity: a file the input tree hands out to be parsed, and what parsing it gave, which only such a file can be given back with.

use std::path::PathBuf;

use ego_tree::NodeId;

use crate::entities::ParsedFeature;
use crate::errors::InputError;

/// Made only by the input tree, which keeps its node.
#[derive(Debug)]
pub(crate) struct FileToParse {
    pub(super) node: NodeId,
    pub(crate) path: PathBuf,
}

#[derive(Debug)]
pub(crate) struct ParsedFile {
    pub(crate) file: FileToParse,
    pub(crate) result: Result<ParsedFeature, Vec<InputError>>,
}
