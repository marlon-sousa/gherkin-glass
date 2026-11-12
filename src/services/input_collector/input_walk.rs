//! Service: the walk of the input into the input tree, listing the folders the `.ghg-ignore` rules allow and marking each entry allowed or not.

use std::io::{Error as IoError, ErrorKind};
use std::path::{Path, PathBuf};

use crate::entities::{
    EntryKind, FolderEntry, IGNORE_FILE, InputTree, InputTreeBuilder, ListedFolder,
};
use crate::errors::{InputError, InputErrorKind};
use crate::policies::{FileSelection, InvalidRule, parse_rules};
use crate::ports::InputFolders;

/// The input tree of the folder `input`, with every error met on the way.
pub(super) fn walk_input(input_folders: &dyn InputFolders, input: &Path) -> InputTree {
    let mut walk = Walk {
        input_folders,
        input,
        selection: FileSelection::new(input),
    };
    match walk.open(input) {
        Opened::Unlisted(error) => InputTreeBuilder::unlisted(input.to_path_buf(), error),
        Opened::Listed(listing) => {
            let mut builder = InputTreeBuilder::listed(input.to_path_buf(), listing.ignore_errors);
            let root = builder.root();
            walk.add_entries(&mut builder, root, listing.entries, listing.entered);
            builder.build()
        }
    }
}

struct Walk<'a> {
    input_folders: &'a dyn InputFolders,
    input: &'a Path,
    selection: FileSelection,
}

/// What listing a folder gave.
enum Opened {
    Unlisted(InputError),
    Listed(Listing),
}

struct Listing {
    entries: Vec<FolderEntry>,
    /// The error met reading the folder's `.ghg-ignore`, or those of its rules that are not valid.
    ignore_errors: Vec<InputError>,
    /// Whether the rules of the folder's `.ghg-ignore` were taken in, to be left once its
    /// entries are added.
    entered: Entered,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Entered {
    Rules,
    NoRules,
}

impl Walk<'_> {
    /// Lists `folder` and takes in the rules of its `.ghg-ignore`, if it has one.
    fn open(&mut self, folder: &Path) -> Opened {
        let entries: Vec<_> = match self.input_folders.list(folder) {
            Ok(entries) => entries.into_iter().filter(is_file_or_folder).collect(),
            Err(error) => {
                let kind = self.listing_error(folder, &error);
                return Opened::Unlisted(self.error(folder, None, kind));
            }
        };
        let mut ignore_errors = Vec::new();
        let entered = if entries.iter().any(is_ignore_file) {
            self.enter_rules(folder, &mut ignore_errors)
        } else {
            Entered::NoRules
        };
        Opened::Listed(Listing {
            entries,
            ignore_errors,
            entered,
        })
    }

    /// Adds `entries`, those of the folder `parent`, each marked allowed or not, then leaves the
    /// folder's rules when `entered`.
    fn add_entries(
        &mut self,
        builder: &mut InputTreeBuilder,
        parent: ListedFolder,
        entries: Vec<FolderEntry>,
        entered: Entered,
    ) {
        for FolderEntry { path, kind } in entries {
            let left_out = self.selection.is_left_out(&path, kind);
            match (kind, left_out) {
                (EntryKind::Folder, true) => builder.add_folder_not_allowed(parent, path),
                (EntryKind::Folder, false) => self.add_folder(builder, parent, path),
                (_, true) => builder.add_file_not_allowed(parent, path),
                (_, false) => builder.add_allowed_file(parent, path),
            }
        }
        if entered == Entered::Rules {
            self.selection.leave();
        }
    }

    fn add_folder(&mut self, builder: &mut InputTreeBuilder, parent: ListedFolder, path: PathBuf) {
        match self.open(&path) {
            Opened::Unlisted(error) => builder.add_unlisted_folder(parent, path, error),
            Opened::Listed(listing) => {
                let folder = builder.add_listed_folder(parent, path, listing.ignore_errors);
                self.add_entries(builder, folder, listing.entries, listing.entered);
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

    /// Reads the rules of `folder`'s `.ghg-ignore` file and takes them in, adding to `errors`
    /// the error met reading it or those of its rules that are not valid.
    fn enter_rules(&mut self, folder: &Path, errors: &mut Vec<InputError>) -> Entered {
        let path = folder.join(IGNORE_FILE);
        let text = match self.input_folders.read(&path) {
            Ok(text) => text,
            Err(error) => {
                let reason = error.to_string();
                errors.push(self.error(&path, None, InputErrorKind::CannotBeRead { reason }));
                return Entered::NoRules;
            }
        };
        let parsed = parse_rules(folder, &path, &text);
        for InvalidRule { line, reason } in parsed.invalid {
            errors.push(self.error(&path, line, InputErrorKind::InvalidRule { reason }));
        }
        self.selection.enter(parsed.rules);
        Entered::Rules
    }

    fn error(&self, path: &Path, line: Option<usize>, kind: InputErrorKind) -> InputError {
        InputError::new(self.input, path, line, kind)
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
        assert_eq!(found.allowed_files(), [PathBuf::from("in/read.feature")]);
        assert!(found.input_errors().is_none());
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
        let errors = found.input_errors().expect("an error");
        let kinds: Vec<_> = errors.as_slice().iter().map(|error| &error.kind).collect();
        assert_eq!(kinds, [&kind]);
    }
}
