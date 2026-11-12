//! Entity: the input tree, the input as the walk found it, each folder and file in it allowed or not, and each allowed file with what parsing it gave once it is parsed.

mod builder;
mod file_to_parse;
mod input_node;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use ego_tree::{NodeId, NodeRef, Tree};

pub(crate) use builder::{InputTreeBuilder, ListedFolder};
pub(crate) use file_to_parse::{FileToParse, ParsedFile};
use input_node::{FileState, InputNode, NodeKind};

use crate::entities::{PageClash, ParsedFeature};
use crate::errors::InputErrors;

/// Its root is the input folder, and each folder's children are in the order of their paths, so
/// that a walk from the root meets the files in file order.
#[derive(Debug, Clone)]
pub(crate) struct InputTree {
    tree: Tree<InputNode>,
}

impl InputTree {
    /// The paths of the allowed files, in file order.
    pub(crate) fn allowed_files(&self) -> Vec<PathBuf> {
        self.tree
            .root()
            .descendants()
            .filter_map(allowed_file)
            .collect()
    }

    /// The allowed files with nothing parsed, in file order.
    pub(crate) fn files_to_parse(&self) -> Vec<FileToParse> {
        self.tree
            .root()
            .descendants()
            .filter_map(file_to_parse)
            .collect()
    }

    /// Puts what parsing each file gave into its node.
    pub(crate) fn put_parsed(&mut self, parsed: Vec<ParsedFile>) {
        for ParsedFile { file, result } in parsed {
            if let Some(mut node) = self.tree.get_mut(file.node) {
                node.value().kind = NodeKind::File(FileState::Parsed(Arc::new(result)));
            }
        }
    }

    /// Puts each clash's error into the node of its file or folder, in place of those the nodes
    /// held; one whose path has no node, which an allowed file or a folder above one always has,
    /// goes into the root's, so that no error is lost.
    pub(crate) fn put_page_errors(&mut self, clashes: Vec<PageClash>) {
        self.tree
            .values_mut()
            .for_each(|node| node.page_errors.clear());
        for PageClash { path, error } in clashes {
            let id = self.node_at(&path).unwrap_or(self.tree.root().id());
            if let Some(mut node) = self.tree.get_mut(id) {
                node.value().page_errors.push(error);
            }
        }
    }

    /// Every error the tree holds, sorted, or nothing when it holds none.
    pub(crate) fn input_errors(&self) -> Option<InputErrors> {
        let errors: Vec<_> = self
            .tree
            .values()
            .flat_map(InputNode::errors)
            .cloned()
            .collect();
        (!errors.is_empty()).then(|| InputErrors::new(errors))
    }

    /// The features of the files parsed without an error, in file order.
    pub(crate) fn features(&self) -> impl Iterator<Item = &ParsedFeature> {
        self.tree.root().descendants().filter_map(parsed_feature)
    }

    fn node_at(&self, path: &Path) -> Option<NodeId> {
        self.tree
            .root()
            .descendants()
            .find(|node| is_at(node, path))
            .map(|node| node.id())
    }
}

fn is_at(node: &NodeRef<'_, InputNode>, path: &Path) -> bool {
    node.value().path == path
}

fn allowed_file(node: NodeRef<'_, InputNode>) -> Option<PathBuf> {
    let InputNode { path, kind, .. } = node.value();
    match kind {
        NodeKind::File(FileState::NothingParsed | FileState::Parsed(_)) => Some(path.clone()),
        NodeKind::File(FileState::NotAllowed) | NodeKind::Folder(_) => None,
    }
}

fn file_to_parse(node: NodeRef<'_, InputNode>) -> Option<FileToParse> {
    let InputNode { path, kind, .. } = node.value();
    matches!(kind, NodeKind::File(FileState::NothingParsed)).then(|| FileToParse {
        node: node.id(),
        path: path.clone(),
    })
}

fn parsed_feature(node: NodeRef<'_, InputNode>) -> Option<&ParsedFeature> {
    match &node.value().kind {
        NodeKind::File(FileState::Parsed(parsed)) => parsed.as_ref().as_ref().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
