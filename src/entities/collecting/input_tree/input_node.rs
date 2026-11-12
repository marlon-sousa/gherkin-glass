//! Entity: a node of the input tree, a folder or a file of the input, as the walk found it.

use std::path::PathBuf;
use std::slice::from_ref;
use std::sync::Arc;

use crate::entities::{IgnoreRules, ParsedFeature, Revision};
use crate::errors::InputError;

#[derive(Debug, Clone)]
pub(super) struct InputNode {
    pub(super) path: PathBuf,
    pub(super) kind: NodeKind,
    /// Those of a file whose page clashes with another, or of a folder whose path differs from
    /// another folder's only in case.
    pub(super) page_errors: Vec<InputError>,
}

#[derive(Debug, Clone)]
pub(super) enum NodeKind {
    Folder(FolderState),
    File(FileState),
}

#[derive(Debug, Clone)]
pub(super) enum FolderState {
    /// Left out by the rules, so never listed, and with no children.
    NotAllowed,
    /// Allowed, but it could not be listed, so it has no children.
    Unlisted(InputError),
    /// Listed, with the rules of its `.ghg-ignore`.
    Listed(IgnoreRules),
}

#[derive(Debug, Clone)]
pub(super) enum FileState {
    NotAllowed,
    NothingParsed,
    /// What parsing gave, shared with the trees made from this one, and the revision the tree
    /// gave the file when it took it in.
    Parsed(Revision, Arc<Result<ParsedFeature, Vec<InputError>>>),
}

impl InputNode {
    pub(super) fn new(path: PathBuf, kind: NodeKind) -> Self {
        InputNode {
            path,
            kind,
            page_errors: Vec::new(),
        }
    }

    /// Every error the node holds: its page's, and a folder's own or those parsing a file gave.
    pub(super) fn errors(&self) -> impl Iterator<Item = &InputError> {
        self.page_errors.iter().chain(self.kind.errors())
    }
}

impl NodeKind {
    fn errors(&self) -> &[InputError] {
        match self {
            NodeKind::Folder(FolderState::Unlisted(error)) => from_ref(error),
            NodeKind::Folder(FolderState::Listed(ignore)) => &ignore.errors,
            NodeKind::File(FileState::Parsed(_, parsed)) => {
                parsed.as_ref().as_ref().err().map_or(&[], Vec::as_slice)
            }
            NodeKind::Folder(FolderState::NotAllowed)
            | NodeKind::File(FileState::NotAllowed | FileState::NothingParsed) => &[],
        }
    }
}
