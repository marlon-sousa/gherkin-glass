//! Service: the input collector, walking the input into the input tree, from nothing or from the last tree and the paths that changed, finding the pages that clash, and saying whether the paths call for a rebuild.

mod input_walk;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use input_walk::{walk_changes, walk_input};

use crate::entities::{CollectedChanges, InputTree};
use crate::policies::{Read, page_collisions};
use crate::ports::InputFolders;

/// What the controller asks of the input collector.
pub(crate) trait InputCollectorApi: Send + Sync {
    /// The input tree of the folder `input`, walked from nothing, with every error met on the way
    /// and those of the pages that clash.
    fn collect(&self, input: &Path) -> InputTree;

    /// The input tree made from `last`, the tree of the last build, and `paths`, which changed in
    /// its input since, with every error and those of the pages that clash, and whether the paths
    /// call for a rebuild.
    fn collect_changes(&self, last: &InputTree, paths: &BTreeSet<PathBuf>) -> CollectedChanges;
}

pub(crate) struct InputCollector {
    input_folders: Arc<dyn InputFolders>,
}

impl InputCollector {
    pub(crate) fn new(input_folders: Arc<dyn InputFolders>) -> Self {
        InputCollector { input_folders }
    }
}

impl InputCollectorApi for InputCollector {
    fn collect(&self, input: &Path) -> InputTree {
        with_page_errors(walk_input(self.input_folders.as_ref(), input))
    }

    fn collect_changes(&self, last: &InputTree, paths: &BTreeSet<PathBuf>) -> CollectedChanges {
        let input_tree = walk_changes(self.input_folders.as_ref(), last, paths);
        let read = Read {
            before: &last.allowed_files(),
            after: &input_tree.allowed_files(),
        };
        CollectedChanges {
            calls_for_rebuild: read.calls_for_rebuild(paths),
            input_tree: with_page_errors(input_tree),
        }
    }
}

/// `input_tree`, with the errors of the pages that clash among its allowed files.
fn with_page_errors(mut input_tree: InputTree) -> InputTree {
    let clashes = page_collisions(input_tree.input_folder(), &input_tree.allowed_files());
    input_tree.put_page_errors(clashes);
    input_tree
}

#[cfg(test)]
mod tests;
