//! Entity: the builder of the input tree, whose typed adds are the only way to put a node in it, so that only a folder that was listed has children, and each folder's children are in the order of their paths, a node copied from the last tree with everything below it.

use std::path::PathBuf;

use ego_tree::{NodeId, NodeRef, Tree};

use super::input_node::{FileState, FolderState, InputNode, NodeKind};
use super::{InputEntry, InputTree};
use crate::entities::{IgnoreRules, Revision};
use crate::errors::InputError;

/// A folder of the tree being built that was listed, so that it may have children.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ListedFolder(NodeId);

pub(crate) struct InputTreeBuilder {
    tree: Tree<InputNode>,
    next_revision: Revision,
}

impl InputTreeBuilder {
    /// A tree whose root, the input folder `input`, was listed, with the rules of its
    /// `.ghg-ignore`, which gives revisions from `next_revision` on.
    pub(crate) fn listed(input: PathBuf, ignore: IgnoreRules, next_revision: Revision) -> Self {
        InputTreeBuilder {
            tree: Tree::new(InputNode::new(
                input,
                NodeKind::Folder(FolderState::Listed(ignore)),
            )),
            next_revision,
        }
    }

    pub(crate) fn root(&self) -> ListedFolder {
        ListedFolder(self.tree.root().id())
    }

    pub(crate) fn add_listed_folder(
        &mut self,
        parent: ListedFolder,
        path: PathBuf,
        ignore: IgnoreRules,
    ) -> ListedFolder {
        let kind = NodeKind::Folder(FolderState::Listed(ignore));
        ListedFolder(self.add(parent, InputNode::new(path, kind)))
    }

    pub(crate) fn add_unlisted_folder(
        &mut self,
        parent: ListedFolder,
        path: PathBuf,
        error: InputError,
    ) {
        let kind = NodeKind::Folder(FolderState::Unlisted(error));
        self.add(parent, InputNode::new(path, kind));
    }

    pub(crate) fn add_folder_not_allowed(&mut self, parent: ListedFolder, path: PathBuf) {
        let kind = NodeKind::Folder(FolderState::NotAllowed);
        self.add(parent, InputNode::new(path, kind));
    }

    /// An allowed file, with nothing parsed.
    pub(crate) fn add_allowed_file(&mut self, parent: ListedFolder, path: PathBuf) {
        let kind = NodeKind::File(FileState::NothingParsed);
        self.add(parent, InputNode::new(path, kind));
    }

    pub(crate) fn add_file_not_allowed(&mut self, parent: ListedFolder, path: PathBuf) {
        let kind = NodeKind::File(FileState::NotAllowed);
        self.add(parent, InputNode::new(path, kind));
    }

    /// `entry` of the last tree, with everything below it, as it was there.
    pub(crate) fn add_unchanged(&mut self, parent: ListedFolder, entry: InputEntry<'_>) {
        let id = self.add(parent, entry.node.value().clone());
        self.copy_children(id, entry);
    }

    pub(crate) fn build(self) -> InputTree {
        InputTree {
            tree: self.tree,
            next_revision: self.next_revision,
        }
    }

    /// The children of `entry`, with everything below them, put after those of `id`, which has
    /// none yet, so that they keep their order.
    fn copy_children(&mut self, id: NodeId, entry: InputEntry<'_>) {
        for child in entry.children() {
            let child_id = self.append(id, child.node.value().clone());
            self.copy_children(child_id, child);
        }
    }

    /// Puts `node` after the children of `parent`.
    #[expect(
        clippy::expect_used,
        reason = "a listed folder is made only by an input tree builder, and each walk uses one"
    )]
    fn append(&mut self, parent: NodeId, node: InputNode) -> NodeId {
        self.tree
            .get_mut(parent)
            .expect("the folder is in the tree")
            .append(node)
            .id()
    }

    /// Puts `node` among the children of `parent` before the first whose path sorts after its own.
    fn add(&mut self, parent: ListedFolder, node: InputNode) -> NodeId {
        let next = self
            .tree
            .get(parent.0)
            .and_then(|parent| first_child_after(parent, &node));
        match next.and_then(|next| self.tree.get_mut(next)) {
            Some(mut next) => next.insert_before(node).id(),
            None => self.append(parent.0, node),
        }
    }
}

fn first_child_after(parent: NodeRef<'_, InputNode>, node: &InputNode) -> Option<NodeId> {
    parent
        .children()
        .find(|child| sorts_after(child, node))
        .map(|child| child.id())
}

fn sorts_after(child: &NodeRef<'_, InputNode>, node: &InputNode) -> bool {
    child.value().path > node.path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_folder_s_children_are_in_the_order_of_their_paths_whatever_order_they_are_added_in() {
        // Rule 15: `a` comes before `a-b.feature`, folder by folder.
        let mut builder =
            InputTreeBuilder::listed("in".into(), IgnoreRules::default(), Revision::default());
        let root = builder.root();
        builder.add_allowed_file(root, "in/a-b.feature".into());
        builder.add_file_not_allowed(root, "in/README.md".into());
        let folder = builder.add_listed_folder(root, "in/a".into(), IgnoreRules::default());
        builder.add_allowed_file(folder, "in/a/y.feature".into());
        builder.add_allowed_file(folder, "in/a/x.feature".into());
        builder.add_folder_not_allowed(root, "in/.git".into());
        let tree = builder.build();
        let paths: Vec<_> = tree.tree.root().descendants().map(shown_path).collect();
        assert_eq!(
            paths,
            [
                "in",
                "in/.git",
                "in/README.md",
                "in/a",
                "in/a/x.feature",
                "in/a/y.feature",
                "in/a-b.feature"
            ]
        );
    }

    fn shown_path(node: NodeRef<'_, InputNode>) -> String {
        node.value().path.display().to_string()
    }
}
