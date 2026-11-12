//! Service: the input collector, walking the input into the input tree and finding the pages that clash.

mod input_walk;

use std::path::Path;
use std::sync::Arc;

use input_walk::walk_input;

use crate::entities::InputTree;
use crate::policies::page_collisions;
use crate::ports::InputFolders;

/// What the controller asks of the input collector.
pub(crate) trait InputCollectorApi: Send + Sync {
    /// The input tree of the folder `input`, walked from nothing, with every error met on the way
    /// and those of the pages that clash.
    fn collect(&self, input: &Path) -> InputTree;
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
}

#[cfg(test)]
mod tests;
