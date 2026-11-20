//! Policy: whether the patterns of what a build leaves alone keep anything ghg writes, or anything in a folder ghg writes.

use std::collections::{BTreeMap, BTreeSet};

use super::{Moment, folder_by_folder, folders_of, made, written};
use crate::entities::{BuildTarget, EntryKind, FilePath, KeptInOutput, Record, VersionName};
use crate::errors::OutputErrorKind;

/// What ghg writes, by its path in the output folder: its files and the folders that hold them.
struct Written {
    files: BTreeSet<FilePath>,
    folders: BTreeSet<FilePath>,
}

/// Nothing, when `kept` keeps nothing ghg writes and `found_kept`, what the
/// folder holds that is kept, is in no folder ghg writes; otherwise the first
/// such thing, folder by folder, with the pattern that keeps it.
pub(super) fn kept_clash(
    kept: &KeptInOutput,
    found_kept: &BTreeMap<FilePath, EntryKind>,
    record: Option<&Record>,
    target: &BuildTarget,
    moment: Moment<'_>,
) -> Result<(), OutputErrorKind> {
    let written = what_ghg_writes(record, target, moment);
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
        .map_or(Ok(()), |clash| Err(clash.into_error()))
}

/// What ghg writes into the output folder: what the marker lists, the marker
/// of `target`, the version's folder, and, when it is about to be written,
/// the site.
fn what_ghg_writes(record: Option<&Record>, target: &BuildTarget, moment: Moment<'_>) -> Written {
    let site = match moment {
        Moment::Writing { site } => Some(site.iter().cloned()),
        Moment::Checking { .. } => None,
    };
    let files: BTreeSet<FilePath> = record
        .map(written)
        .unwrap_or_default()
        .into_iter()
        .chain([marker_of(target).to_string()])
        .chain(site.into_iter().flatten())
        .collect();
    let recorded = record
        .map(|record| made(record, &files))
        .unwrap_or_default();
    let folders = files
        .iter()
        .flat_map(|file| folders_of(file))
        .chain(recorded)
        .chain(version_folder(target))
        .collect();
    Written { files, folders }
}

impl Written {
    /// Whether `path` is something ghg writes, or is in a folder ghg writes.
    fn holds(&self, path: &str) -> bool {
        self.files.contains(path)
            || self.folders.contains(path)
            || folders_of(path).any(|folder| self.folders.contains(&folder))
    }
}

fn marker_of(target: &BuildTarget) -> &'static str {
    match target {
        BuildTarget::Site => Record::SITE_MARKER,
        BuildTarget::Version(_) => Record::VERSIONS_MARKER,
    }
}

fn version_folder(target: &BuildTarget) -> Option<FilePath> {
    match target {
        BuildTarget::Site => None,
        BuildTarget::Version(name) => Some(VersionName::to_string(name)),
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

/// The clash at `path`, when `kept` keeps it.
fn clash_at(kept: &KeptInOutput, path: &str, kind: EntryKind) -> Option<Clash> {
    let pattern = kept.keeping(path, kind)?;
    Some(Clash {
        found: path.to_string(),
        pattern: pattern.as_str().to_string(),
    })
}

#[cfg(test)]
mod tests;
