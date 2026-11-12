//! Service: what the last input tree and the paths that changed since say of a folder the walk lists: its entry there, what of it is kept, and whether its rules or those of a folder it is in changed.

use std::collections::{BTreeSet, HashMap};
use std::ops::Bound;
use std::path::{Path, PathBuf};

use crate::entities::{IGNORE_FILE, IgnoreRules, InputEntry};

/// The paths that changed since the last tree, none for a walk from nothing.
#[derive(Clone, Copy)]
pub(super) struct Changed<'a> {
    pub(super) paths: &'a BTreeSet<PathBuf>,
}

impl Changed<'_> {
    pub(super) fn names(&self, path: &Path) -> bool {
        self.paths.contains(path)
    }

    /// Whether one of the paths is `path` or is in it. The paths in a folder sort together, from
    /// the folder itself on.
    pub(super) fn concerns(&self, path: &Path) -> bool {
        self.paths
            .range::<Path, _>((Bound::Included(path), Bound::Unbounded))
            .next()
            .is_some_and(|first| first.starts_with(path))
    }

    fn names_rules_of(&self, folder: &Path) -> bool {
        self.names(&folder.join(IGNORE_FILE))
    }
}

/// What the last tree holds of a folder being listed: its entry, if it is the same folder, and
/// whether the rules of a folder it is in or of its own changed, so that no folder below it is
/// copied whole.
#[derive(Clone, Copy)]
pub(super) struct Last<'a> {
    entry: Option<InputEntry<'a>>,
    rules_changed: bool,
}

impl<'a> Last<'a> {
    /// What the last tree holds of `folder`, whose entry there is `entry`, below a folder whose
    /// rules changed when `rules_changed_above`. A folder a path names may have been replaced by
    /// another at the same path, whose files keep their names, so it keeps no entry.
    pub(super) fn of(
        folder: &Path,
        entry: Option<InputEntry<'a>>,
        rules_changed_above: bool,
        changed: Changed<'_>,
    ) -> Self {
        Last {
            entry: entry.filter(|_| !changed.names(folder)),
            rules_changed: rules_changed_above || changed.names_rules_of(folder),
        }
    }

    pub(super) fn rules_changed(&self) -> bool {
        self.rules_changed
    }

    /// The rules the folder's node kept, unless its `.ghg-ignore` is among the paths.
    pub(super) fn kept_rules(
        &self,
        folder: &Path,
        changed: Changed<'_>,
    ) -> Option<&'a IgnoreRules> {
        let entry = self.entry.filter(|_| !changed.names_rules_of(folder))?;
        entry.listed_rules()
    }

    /// The entries of the folder in the last tree, by their paths.
    pub(super) fn children(&self) -> HashMap<&'a Path, InputEntry<'a>> {
        self.entry
            .iter()
            .flat_map(InputEntry::children)
            .map(keyed_by_path)
            .collect()
    }

    /// Whether `kept`, a folder of the last tree in this one, is copied whole: one that was
    /// listed, which no path concerns, under no rules that changed.
    pub(super) fn keeps_folder(&self, kept: InputEntry<'_>, changed: Changed<'_>) -> bool {
        !self.rules_changed && kept.listed_rules().is_some() && !changed.concerns(kept.path())
    }
}

/// Whether `kept`, an entry of the last tree at `path`, is copied as a file: one the rules
/// allowed, which no path names.
pub(super) fn keeps_file(kept: InputEntry<'_>, path: &Path, changed: Changed<'_>) -> bool {
    kept.is_allowed_file() && !changed.names(path)
}

fn keyed_by_path(entry: InputEntry<'_>) -> (&Path, InputEntry<'_>) {
    (entry.path(), entry)
}
