//! Entity: an entry of an input tree, a folder or a file with everything below it, as the walk that makes the next tree from it reads it.

use std::path::Path;

use ego_tree::NodeRef;

use super::input_node::{FileState, FolderState, InputNode, NodeKind};
use crate::entities::IgnoreRules;

#[derive(Debug, Clone, Copy)]
pub(crate) struct InputEntry<'a> {
    pub(super) node: NodeRef<'a, InputNode>,
}

impl<'a> InputEntry<'a> {
    pub(crate) fn path(&self) -> &'a Path {
        &self.node.value().path
    }

    /// The entries directly below it, in the order of their paths.
    pub(crate) fn children(&self) -> impl Iterator<Item = InputEntry<'a>> + use<'a> {
        self.node.children().map(|node| InputEntry { node })
    }

    /// The rules of its `.ghg-ignore`, when it is a folder that was listed.
    pub(crate) fn listed_rules(&self) -> Option<&'a IgnoreRules> {
        match &self.node.value().kind {
            NodeKind::Folder(FolderState::Listed(ignore)) => Some(ignore),
            _ => None,
        }
    }

    /// Whether it is a file the rules allowed.
    pub(crate) fn is_allowed_file(&self) -> bool {
        matches!(
            self.node.value().kind,
            NodeKind::File(FileState::NothingParsed | FileState::Parsed(..))
        )
    }
}
