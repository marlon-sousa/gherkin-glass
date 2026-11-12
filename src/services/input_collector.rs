//! Service: the input collector, walking the input into the input tree, finding the pages that clash, and saying which changes under `--watch` call for a rebuild.

mod input_walk;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use input_walk::walk_input;

use crate::entities::InputTree;
use crate::policies::{Read, page_collisions};
use crate::ports::InputFolders;

/// What the controller asks of the input collector.
pub(crate) trait InputCollectorApi: Send + Sync {
    /// The input tree of the folder `input`, walked from nothing, with every error met on the way
    /// and those of the pages that clash.
    fn collect(&self, input: &Path) -> InputTree;

    /// Whether `paths`, which changed in the input of `last`, the tree of the last build, call
    /// for a rebuild, judged against the input walked again from nothing.
    fn calls_for_rebuild(&self, last: &InputTree, paths: &BTreeSet<PathBuf>) -> bool;
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
        let mut input_tree = walk_input(self.input_folders.as_ref(), input);
        let clashes = page_collisions(input, &input_tree.allowed_files());
        input_tree.put_page_errors(clashes);
        input_tree
    }

    fn calls_for_rebuild(&self, last: &InputTree, paths: &BTreeSet<PathBuf>) -> bool {
        let walked = walk_input(self.input_folders.as_ref(), last.input_folder());
        let read = Read {
            before: &last.allowed_files(),
            after: &walked.allowed_files(),
        };
        read.calls_for_rebuild(paths)
    }
}

#[cfg(test)]
mod tests;
