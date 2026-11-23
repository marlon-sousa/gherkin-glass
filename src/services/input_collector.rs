//! Service: the input collector, reading the list `--only` names and walking the input, or the files the list holds in it, into the input tree, from nothing or from the last tree and the paths that changed, finding the pages that clash, and saying whether the paths call for a rebuild.

mod input_walk;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use input_walk::{walk_changes, walk_input};

use crate::entities::{CollectedChanges, InputTree, ListedFiles, PathList};
use crate::errors::ConfigErrors;
use crate::policies::{ListedSelection, Read, listed_paths, page_collisions};
use crate::ports::{FileReader, InputFolders, StandardInput};

/// What the controller asks of the input collector.
pub(crate) trait InputCollectorApi: Send + Sync {
    /// The input tree of the folder `input`, walked from nothing, with every error met on the way
    /// and those of the pages that clash, of the files the list `only` holds, or of every file
    /// when it is `None`; or the error of the list when it cannot be read.
    fn collect(&self, input: &Path, only: Option<&PathList>) -> Result<InputTree, ConfigErrors>;

    /// The input tree made from `last`, the tree of the last build, and `paths`, which changed in
    /// its input since, with every error and those of the pages that clash, and whether the paths
    /// call for a rebuild, of the files `last` was walked for. The list is not read again: a
    /// change to its file is a change of everything, which builds from nothing.
    fn collect_changes(&self, last: &InputTree, paths: &BTreeSet<PathBuf>) -> CollectedChanges;
}

pub(crate) struct InputCollector {
    input_folders: Arc<dyn InputFolders>,
    list_reader: Arc<dyn FileReader>,
    standard_input: Arc<dyn StandardInput>,
}

impl InputCollector {
    /// `list_reader` and `standard_input` read the list `--only` names, from its file or from
    /// standard input.
    pub(crate) fn new(
        input_folders: Arc<dyn InputFolders>,
        list_reader: Arc<dyn FileReader>,
        standard_input: Arc<dyn StandardInput>,
    ) -> Self {
        InputCollector {
            input_folders,
            list_reader,
            standard_input,
        }
    }

    /// What the list `only` names inside the folder `input`, `None` without a list, or the
    /// error of the list when it cannot be read.
    fn listed_files(
        &self,
        input: &Path,
        only: Option<&PathList>,
    ) -> Result<Option<ListedFiles>, ConfigErrors> {
        let Some(list) = only else {
            return Ok(None);
        };
        let text = match list {
            PathList::File(file) => self.list_reader.read(file),
            PathList::StandardInput => self.standard_input.read(),
        }
        .map_err(|error| ConfigErrors::unreadable(list.as_given(), &error))?;
        let paths = listed_paths(&text);
        if paths.is_empty() {
            return Ok(Some(ListedFiles::NoPath));
        }
        let selection = ListedSelection::new(
            &self.absolute(input),
            paths.iter().map(|path| self.absolute(path)),
        );
        Ok(Some(ListedFiles::Below(selection.into_files())))
    }

    /// `path` from the current folder, or as written when the current folder cannot be found.
    fn absolute(&self, path: &Path) -> PathBuf {
        self.input_folders
            .absolute(path)
            .unwrap_or_else(|_| path.to_path_buf())
    }
}

impl InputCollectorApi for InputCollector {
    fn collect(&self, input: &Path, only: Option<&PathList>) -> Result<InputTree, ConfigErrors> {
        let listed = self.listed_files(input, only)?;
        let selection = listed
            .as_ref()
            .map(|listed| ListedSelection::of(listed.files()));
        let input_tree = walk_input(self.input_folders.as_ref(), input, selection);
        Ok(with_page_errors(input_tree.walked_for(listed)))
    }

    fn collect_changes(&self, last: &InputTree, paths: &BTreeSet<PathBuf>) -> CollectedChanges {
        let listed = last
            .listed()
            .map(|listed| ListedSelection::of(listed.files()));
        let input_tree = walk_changes(self.input_folders.as_ref(), last, paths, listed)
            .walked_for(last.listed().cloned());
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
