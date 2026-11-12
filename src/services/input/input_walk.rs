//! Service: the walk of the input, entering the folders and keeping the files that the `.ghg-ignore` rules select.

use std::io::{Error as IoError, ErrorKind};
use std::path::Path;

use crate::entities::{CollectedInput, EntryKind, FolderEntry};
use crate::errors::{InputError, InputErrorKind};
use crate::policies::{FileSelection, IGNORE_FILE, InvalidRule, parse_rules};
use crate::ports::InputFolders;

/// The files under `input` that the rules select, sorted, and every error met on the way.
pub(super) fn walk_input(folders: &dyn InputFolders, input: &Path) -> CollectedInput {
    let mut walk = Walk {
        folders,
        input,
        selection: FileSelection::new(input),
        found: CollectedInput::default(),
    };
    walk.visit(input);
    walk.found.paths.sort();
    walk.found
}

struct Walk<'a> {
    folders: &'a dyn InputFolders,
    input: &'a Path,
    selection: FileSelection,
    found: CollectedInput,
}

impl Walk<'_> {
    fn visit(&mut self, folder: &Path) {
        let Some(entries) = self.entries(folder) else {
            return;
        };
        let entered = entries.iter().any(is_ignore_file) && self.enter_rules(folder);
        for FolderEntry { path, kind } in entries {
            if self.selection.is_left_out(&path, kind) {
                continue;
            }
            if kind == EntryKind::Folder {
                self.visit(&path);
            } else {
                self.found.paths.push(path);
            }
        }
        if entered {
            self.selection.leave();
        }
    }

    /// The files and folders in `folder`.
    fn entries(&mut self, folder: &Path) -> Option<Vec<FolderEntry>> {
        match self.folders.list(folder) {
            Ok(entries) => Some(entries.into_iter().filter(is_file_or_folder).collect()),
            Err(error) => {
                let kind = self.listing_error(folder, &error);
                self.add_error(folder, None, kind);
                None
            }
        }
    }

    fn listing_error(&self, folder: &Path, error: &IoError) -> InputErrorKind {
        if folder == self.input && error.kind() == ErrorKind::NotFound {
            InputErrorKind::InputMissing
        } else {
            InputErrorKind::CannotBeListed {
                reason: error.to_string(),
            }
        }
    }

    /// Whether the rules of `folder`'s `.ghg-ignore` file were taken in.
    fn enter_rules(&mut self, folder: &Path) -> bool {
        let path = folder.join(IGNORE_FILE);
        let text = match self.folders.read(&path) {
            Ok(text) => text,
            Err(error) => {
                let reason = error.to_string();
                self.add_error(&path, None, InputErrorKind::CannotBeRead { reason });
                return false;
            }
        };
        let parsed = parse_rules(folder, &path, &text);
        for InvalidRule { line, reason } in parsed.invalid {
            self.add_error(&path, line, InputErrorKind::InvalidRule { reason });
        }
        self.selection.enter(parsed.rules);
        true
    }

    fn add_error(&mut self, path: &Path, line: Option<usize>, kind: InputErrorKind) {
        self.found
            .errors
            .push(InputError::new(self.input, path, line, kind));
    }
}

fn is_file_or_folder(entry: &FolderEntry) -> bool {
    entry.kind != EntryKind::Other
}

fn is_ignore_file(entry: &FolderEntry) -> bool {
    entry.kind == EntryKind::File && entry.path.file_name() == Some(IGNORE_FILE.as_ref())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::io::Result as IoResult;
    use std::path::PathBuf;

    use rstest::rstest;

    use super::*;

    /// The entries of each folder it holds; any other folder is not found.
    struct Listings {
        folders: BTreeMap<PathBuf, Vec<FolderEntry>>,
    }

    impl InputFolders for Listings {
        fn list(&self, folder: &Path) -> IoResult<Vec<FolderEntry>> {
            self.folders
                .get(folder)
                .cloned()
                .ok_or_else(|| IoError::new(ErrorKind::NotFound, "not found"))
        }

        fn read(&self, _: &Path) -> IoResult<String> {
            Err(IoError::new(ErrorKind::NotFound, "not found"))
        }
    }

    fn entry(path: &str, kind: EntryKind) -> FolderEntry {
        FolderEntry {
            path: PathBuf::from(path),
            kind,
        }
    }

    fn listings(folders: Vec<(&str, Vec<FolderEntry>)>) -> Listings {
        Listings {
            folders: folders.into_iter().map(listing).collect(),
        }
    }

    fn listing((folder, entries): (&str, Vec<FolderEntry>)) -> (PathBuf, Vec<FolderEntry>) {
        (PathBuf::from(folder), entries)
    }

    #[test]
    fn an_entry_that_is_neither_a_file_nor_a_folder_is_left_out() {
        // Rule 5: a symbolic link is listed as neither, and is not followed.
        let folders = listings(vec![(
            "in",
            vec![
                entry("in/linked.feature", EntryKind::Other),
                entry("in/read.feature", EntryKind::File),
            ],
        )]);
        let found = walk_input(&folders, Path::new("in"));
        assert_eq!(found.paths, [PathBuf::from("in/read.feature")]);
        assert!(found.errors.is_empty());
    }

    #[rstest]
    // Rules 14 and 16.
    #[case::the_input_itself(Vec::new(), InputErrorKind::InputMissing)]
    #[case::a_folder_inside_it(
        vec![("in", vec![entry("in/gone", EntryKind::Folder)])],
        InputErrorKind::CannotBeListed { reason: "not found".to_string() },
    )]
    fn only_the_input_itself_not_found_is_a_missing_input(
        #[case] folders: Vec<(&str, Vec<FolderEntry>)>,
        #[case] kind: InputErrorKind,
    ) {
        let found = walk_input(&listings(folders), Path::new("in"));
        let kinds: Vec<_> = found.errors.into_iter().map(|error| error.kind).collect();
        assert_eq!(kinds, [kind]);
    }
}
