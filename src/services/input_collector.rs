//! Service: the input collector, walking the input, or the files `--only` lists in it, into the input tree, from nothing or from the last tree and the paths that changed, finding the pages that clash, and saying whether the paths call for a rebuild.

mod input_walk;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use input_walk::{walk_changes, walk_input};

use crate::entities::{CollectedChanges, InputTree, ListedFiles};
use crate::policies::{ListedSelection, Read, page_collisions};
use crate::ports::InputFolders;

/// What the controller asks of the input collector.
pub(crate) trait InputCollectorApi: Send + Sync {
    /// The input tree of the folder `input`, walked from nothing, with every error met on the way
    /// and those of the pages that clash, of the files `only` lists, or of every file when it is
    /// `None`.
    fn collect(&self, input: &Path, only: Option<&ListedFiles>) -> InputTree;

    /// The input tree made from `last`, the tree of the last build, and `paths`, which changed in
    /// its input since, with every error and those of the pages that clash, and whether the paths
    /// call for a rebuild, of the files `only` lists, as `last` was.
    fn collect_changes(
        &self,
        last: &InputTree,
        paths: &BTreeSet<PathBuf>,
        only: Option<&ListedFiles>,
    ) -> CollectedChanges;
}

pub(crate) struct InputCollector {
    input_folders: Arc<dyn InputFolders>,
}

impl InputCollector {
    pub(crate) fn new(input_folders: Arc<dyn InputFolders>) -> Self {
        InputCollector { input_folders }
    }

    /// The files of `only` inside the folder `input`.
    fn listed_selection(&self, input: &Path, only: &ListedFiles) -> ListedSelection {
        ListedSelection::new(
            &self.absolute(input),
            only.paths.iter().map(|path| self.absolute(path)),
        )
    }

    /// `path` from the current folder, or as written when the current folder cannot be found.
    fn absolute(&self, path: &Path) -> PathBuf {
        self.input_folders
            .absolute(path)
            .unwrap_or_else(|_| path.to_path_buf())
    }
}

impl InputCollectorApi for InputCollector {
    fn collect(&self, input: &Path, only: Option<&ListedFiles>) -> InputTree {
        let listed = only.map(|only| self.listed_selection(input, only));
        with_page_errors(walk_input(self.input_folders.as_ref(), input, listed))
    }

    fn collect_changes(
        &self,
        last: &InputTree,
        paths: &BTreeSet<PathBuf>,
        only: Option<&ListedFiles>,
    ) -> CollectedChanges {
        let input = last.input_folder();
        let listed = only.map(|only| self.listed_selection(input, only));
        let input_tree = walk_changes(self.input_folders.as_ref(), last, paths, listed);
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
