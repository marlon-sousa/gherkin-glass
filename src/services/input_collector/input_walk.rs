//! Service: the walk of the input into the input tree, from nothing or from the last tree and the paths that changed, listing the folders the `.ghg-ignore` rules allow, marking each entry allowed or not, and copying from the last tree what the paths do not concern.

use std::collections::{BTreeSet, HashMap};
use std::io::{Error as IoError, ErrorKind};
use std::ops::Bound;
use std::path::{Path, PathBuf};

use crate::entities::{
    EntryKind, FolderEntry, IGNORE_FILE, IgnoreRules, InputEntry, InputTree, InputTreeBuilder,
    ListedFolder, Revision,
};
use crate::errors::{InputError, InputErrorKind};
use crate::policies::{FileSelection, InvalidRule, parse_rules};
use crate::ports::InputFolders;

/// The input tree of the folder `input`, with every error met on the way.
pub(super) fn walk_input(input_folders: &dyn InputFolders, input: &Path) -> InputTree {
    let changed = BTreeSet::new();
    Walk::new(input_folders, input, &changed, Revision::default()).tree(None)
}

/// The input tree made from `last` and the paths `changed` in its input since: a folder is
/// listed again when one of them is in it or is it, and walked as from nothing below a
/// `.ghg-ignore` among them, a file among them has nothing parsed, and every other node is
/// copied from `last`, with what parsing gave.
pub(super) fn walk_changes(
    input_folders: &dyn InputFolders,
    last: &InputTree,
    changed: &BTreeSet<PathBuf>,
) -> InputTree {
    let input = last.input_folder();
    if !concerns(changed, input) {
        return last.clone();
    }
    Walk::new(input_folders, input, changed, last.next_revision()).tree(Some(last.input_entry()))
}

/// Whether one of `changed` is `path` or is in it. The paths in a folder sort together, from
/// the folder itself on.
fn concerns(changed: &BTreeSet<PathBuf>, path: &Path) -> bool {
    changed
        .range::<Path, _>((Bound::Included(path), Bound::Unbounded))
        .next()
        .is_some_and(|first| first.starts_with(path))
}

struct Walk<'a> {
    input_folders: &'a dyn InputFolders,
    input: &'a Path,
    selection: FileSelection,
    changed: &'a BTreeSet<PathBuf>,
    next_revision: Revision,
}

/// What listing a folder gave.
enum Opened {
    Unlisted(InputError),
    Listed(Listing),
}

struct Listing {
    entries: Vec<FolderEntry>,
    ignore: IgnoreRules,
    /// Whether the rules of the folder's `.ghg-ignore` were taken in, to be left once its
    /// entries are added.
    entered: Entered,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Entered {
    Rules,
    NoRules,
}

/// What the last tree holds of a folder being listed: its entry, if it has one, and whether the
/// rules of a folder it is in or of its own changed, so that nothing below it is copied.
#[derive(Clone, Copy)]
struct Last<'a> {
    entry: Option<InputEntry<'a>>,
    rules_changed: bool,
}

impl<'a> Walk<'a> {
    fn new(
        input_folders: &'a dyn InputFolders,
        input: &'a Path,
        changed: &'a BTreeSet<PathBuf>,
        next_revision: Revision,
    ) -> Self {
        Walk {
            input_folders,
            input,
            selection: FileSelection::new(input),
            changed,
            next_revision,
        }
    }

    /// The tree of the input folder, whose entry in the last tree is `last`, if there is one.
    fn tree(mut self, last: Option<InputEntry<'_>>) -> InputTree {
        let input = self.input;
        let last = self.last_of(input, last, false);
        match self.open(input, last) {
            Opened::Unlisted(error) => {
                InputTree::unlisted_input(input.to_path_buf(), error, self.next_revision)
            }
            Opened::Listed(listing) => {
                let mut builder = InputTreeBuilder::listed(
                    input.to_path_buf(),
                    listing.ignore,
                    self.next_revision,
                );
                let root = builder.root();
                self.add_entries(&mut builder, root, listing.entries, listing.entered, last);
                builder.build()
            }
        }
    }

    /// Lists `folder`, and takes in the rules of its `.ghg-ignore`, if it has one: those `last`
    /// kept, unless its `.ghg-ignore` changed, or else read again.
    fn open(&mut self, folder: &Path, last: Last<'_>) -> Opened {
        let entries: Vec<_> = match self.input_folders.list(folder) {
            Ok(entries) => entries.into_iter().filter(is_file_or_folder).collect(),
            Err(error) => {
                let kind = self.listing_error(folder, &error);
                return Opened::Unlisted(self.error(folder, None, kind));
            }
        };
        let kept = last.entry.and_then(|entry| entry.listed_rules());
        let ignore = match kept {
            Some(kept) if !last.rules_changed => kept.clone(),
            _ if entries.iter().any(is_ignore_file) => self.read_rules(folder),
            _ => IgnoreRules::default(),
        };
        let entered = match &ignore.rules {
            Some(rules) => {
                self.selection.enter(rules.clone());
                Entered::Rules
            }
            None => Entered::NoRules,
        };
        Opened::Listed(Listing {
            entries,
            ignore,
            entered,
        })
    }

    /// Adds `entries`, those of the folder `parent`, each marked allowed or not, copying those of
    /// `last` that no changed path concerns, then leaves the folder's rules when `entered`.
    fn add_entries(
        &mut self,
        builder: &mut InputTreeBuilder,
        parent: ListedFolder,
        entries: Vec<FolderEntry>,
        entered: Entered,
        last: Last<'_>,
    ) {
        let kept: HashMap<_, _> = last
            .entry
            .iter()
            .flat_map(InputEntry::children)
            .map(|entry| (entry.path(), entry))
            .collect();
        for FolderEntry { path, kind } in entries {
            let left_out = self.selection.is_left_out(&path, kind);
            let kept = kept.get(path.as_path()).copied();
            match (kind, left_out) {
                (EntryKind::Folder, true) => builder.add_folder_not_allowed(parent, path),
                (EntryKind::Folder, false) => match kept {
                    Some(kept) if self.is_unchanged_folder(kept, last) => {
                        builder.add_unchanged(parent, kept);
                    }
                    _ => {
                        let last = self.last_of(&path, kept, last.rules_changed);
                        self.add_folder(builder, parent, path, last);
                    }
                },
                (_, true) => builder.add_file_not_allowed(parent, path),
                (_, false) => match kept {
                    Some(kept) if kept.is_allowed_file() && !self.changed.contains(&path) => {
                        builder.add_unchanged(parent, kept);
                    }
                    _ => builder.add_allowed_file(parent, path),
                },
            }
        }
        if entered == Entered::Rules {
            self.selection.leave();
        }
    }

    fn add_folder(
        &mut self,
        builder: &mut InputTreeBuilder,
        parent: ListedFolder,
        path: PathBuf,
        last: Last<'_>,
    ) {
        match self.open(&path, last) {
            Opened::Unlisted(error) => builder.add_unlisted_folder(parent, path, error),
            Opened::Listed(listing) => {
                let folder = builder.add_listed_folder(parent, path, listing.ignore);
                self.add_entries(builder, folder, listing.entries, listing.entered, last);
            }
        }
    }

    /// What the last tree holds of `folder`, whose entry there is `entry`, below a folder whose
    /// rules changed when `rules_changed_above`.
    fn last_of<'e>(
        &self,
        folder: &Path,
        entry: Option<InputEntry<'e>>,
        rules_changed_above: bool,
    ) -> Last<'e> {
        Last {
            entry,
            rules_changed: rules_changed_above || self.changed.contains(&folder.join(IGNORE_FILE)),
        }
    }

    /// Whether `kept`, a folder of the last tree in the folder `last` holds, is copied: one that
    /// was listed, which no changed path concerns, under no rules that changed.
    fn is_unchanged_folder(&self, kept: InputEntry<'_>, last: Last<'_>) -> bool {
        !last.rules_changed && kept.listed_rules().is_some() && !concerns(self.changed, kept.path())
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

    /// Reads the rules of `folder`'s `.ghg-ignore` file, with the error met reading it or those
    /// of its rules that are not valid.
    fn read_rules(&self, folder: &Path) -> IgnoreRules {
        let path = folder.join(IGNORE_FILE);
        let text = match self.input_folders.read(&path) {
            Ok(text) => text,
            Err(error) => {
                let reason = error.to_string();
                let kind = InputErrorKind::CannotBeRead { reason };
                return IgnoreRules {
                    rules: None,
                    errors: vec![self.error(&path, None, kind)],
                };
            }
        };
        let parsed = parse_rules(folder, &path, &text);
        let errors = parsed
            .invalid
            .into_iter()
            .map(|rule| self.invalid_rule(&path, rule))
            .collect();
        IgnoreRules {
            rules: Some(parsed.rules),
            errors,
        }
    }

    fn invalid_rule(&self, path: &Path, InvalidRule { line, reason }: InvalidRule) -> InputError {
        self.error(path, line, InputErrorKind::InvalidRule { reason })
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
