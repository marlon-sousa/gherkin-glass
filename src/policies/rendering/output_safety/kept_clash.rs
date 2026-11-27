//! Policy: whether the patterns of what a command leaves alone keep anything ghg writes, or anything in a folder ghg writes.

use std::collections::{BTreeMap, BTreeSet};

use super::{Moment, folders_of};
use crate::entities::{BuildTarget, EntryKind, FilePath, KeptInOutput, LetterCase, OutputMarker};
use crate::errors::OutputErrorKind;
use crate::policies::rendering::{PAGES_INDEX, ROOT_PAGE, VERSIONS_LIST, folder_by_folder};

/// What ghg writes, by its path in the output folder: its files and the folders that hold them.
struct Written {
    files: BTreeSet<FilePath>,
    folders: BTreeSet<FilePath>,
    /// Every path of `files` and `folders`, in small letters.
    lowered: BTreeSet<String>,
}

/// Nothing, when `kept` keeps nothing ghg writes and `found_kept`, what the folder holds that is
/// kept, is in no folder ghg writes; otherwise the first such thing, folder by folder, with the
/// pattern that keeps it.
pub(super) fn kept_clash(
    kept: &KeptInOutput,
    found_kept: &BTreeMap<FilePath, EntryKind>,
    listed: &BTreeSet<FilePath>,
    target: &BuildTarget,
    moment: Moment<'_>,
) -> Result<(), OutputErrorKind> {
    let written = what_ghg_writes(listed, target, moment);
    let files = written
        .files
        .iter()
        .filter_map(|file| clash_at(kept, file, EntryKind::File));
    let folders = written
        .folders
        .iter()
        .filter_map(|folder| clash_at(kept, folder, EntryKind::Folder));
    let found = found_kept
        .iter()
        .filter(|(path, _)| written.holds(path))
        .filter_map(|(path, kind)| clash_at(kept, path, *kind));
    files
        .chain(folders)
        .chain(found)
        .min_by(|one, other| folder_by_folder(&one.found, &other.found))
        .map(Clash::into_error)
        .map_or(Ok(()), Err)
}

/// What ghg writes into the output folder: what the marker lists, the marker, the target's
/// folder, and, when it is about to be written, the target, with the files made from the
/// descriptors of versions beside a version.
fn what_ghg_writes(
    listed: &BTreeSet<FilePath>,
    target: &BuildTarget,
    moment: Moment<'_>,
) -> Written {
    let derived = match target {
        BuildTarget::Version(_) => [VERSIONS_LIST, PAGES_INDEX, ROOT_PAGE]
            .map(str::to_string)
            .to_vec(),
        BuildTarget::Site | BuildTarget::Preview(_) => Vec::new(),
    };
    let site = match moment {
        Moment::Writing { site } => Some(site.iter().cloned().chain(derived)),
        Moment::Checking { .. } => None,
    };
    let files: BTreeSet<FilePath> = listed
        .iter()
        .cloned()
        .chain([OutputMarker::FILE.to_string()])
        .chain(site.into_iter().flatten())
        .collect();
    let target_folder = target.folder();
    let folders: BTreeSet<FilePath> = files
        .iter()
        .chain(&target_folder)
        .flat_map(|file| folders_of(file))
        .chain(target_folder.clone())
        .collect();
    let lowered = files
        .iter()
        .chain(&folders)
        .map(|path| path.to_lowercase())
        .collect();
    Written {
        files,
        folders,
        lowered,
    }
}

impl Written {
    /// Whether `path` is something ghg writes, or is in a folder ghg writes, whatever the case of
    /// its letters, as a file system that does not tell capitals apart would find it.
    fn holds(&self, path: &str) -> bool {
        let path = path.to_lowercase();
        self.lowered.contains(&path)
            || folders_of(&path).any(|folder| self.lowered.contains(&folder))
    }
}

/// Something ghg writes, or something in a folder ghg writes, that a pattern keeps.
struct Clash {
    found: FilePath,
    pattern: String,
}

impl Clash {
    fn into_error(self) -> OutputErrorKind {
        OutputErrorKind::KeptClash {
            found: self.found,
            pattern: self.pattern,
        }
    }
}

/// The clash at `path`, when `kept` keeps it as its patterns are written or whatever the case of
/// their letters: either way, a file system could keep it.
fn clash_at(kept: &KeptInOutput, path: &str, kind: EntryKind) -> Option<Clash> {
    let pattern = kept
        .keeping(path, kind, LetterCase::AsWritten)
        .or_else(|| kept.keeping(path, kind, LetterCase::Whatever))?;
    Some(Clash {
        found: path.to_string(),
        pattern: pattern.as_str().to_string(),
    })
}

#[cfg(test)]
mod tests;
