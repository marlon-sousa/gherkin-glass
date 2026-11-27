//! Policy: whether a build may write into an output folder, from what the folder holds, what its marker records, what the build leaves alone there, and when it is looked at.

mod kept_clash;

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use crate::entities::{
    BuildTarget, EntryKind, FilePath, KeptInOutput, OutputContents, Record, VersionName,
};
use crate::errors::OutputErrorKind;

use kept_clash::kept_clash;

/// When a folder is looked at: when it is checked, before the input is read, or again just
/// before it is written, once the input has been read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Moment<'a> {
    /// `replace` says whether a build already there may be replaced.
    Checking { replace: bool },
    /// `site` is every file the build writes, by its path in the output folder.
    Writing { site: &'a BTreeSet<FilePath> },
}

/// What the output folder's marker records, `None` when the folder is
/// missing or holds only what `kept` keeps, once nothing stops `target` being
/// written into it; or what stops it. A folder ghg marked must hold only what
/// it wrote there and what `kept` keeps, which must be nothing ghg writes.
pub(crate) fn output_safety(
    contents: OutputContents,
    target: &BuildTarget,
    moment: Moment<'_>,
    kept: &KeptInOutput,
) -> Result<Option<Record>, OutputErrorKind> {
    let (files, folders, found_kept, record) = match contents {
        OutputContents::Missing => (BTreeSet::new(), BTreeSet::new(), BTreeMap::new(), None),
        OutputContents::NotAFolder => return Err(OutputErrorKind::NotASite),
        OutputContents::Folder {
            files,
            folders,
            kept,
            record,
        } => (files, folders, kept, record),
    };
    let tops: BTreeSet<_> = files.iter().chain(&folders).filter_map(top_of).collect();
    let record = marked(&files, &folders, &found_kept, record, target, moment)?;
    kept_clash(kept, &found_kept, record.as_ref(), target, moment)?;
    Ok(record.map(|record| found_only(record, &files, &tops)))
}

/// What the marker records, `None` when the folder holds nothing but what is
/// kept, once nothing but what is kept stops `target` being written into it.
fn marked(
    files: &BTreeSet<FilePath>,
    folders: &BTreeSet<FilePath>,
    found_kept: &BTreeMap<FilePath, EntryKind>,
    record: Option<Record>,
    target: &BuildTarget,
    moment: Moment<'_>,
) -> Result<Option<Record>, OutputErrorKind> {
    // A folder holding what is kept is no stray, though what else it holds may be.
    let holding: BTreeSet<_> = found_kept
        .keys()
        .flat_map(|path| folders_of(path))
        .collect();
    let folders: BTreeSet<_> = folders.difference(&holding).cloned().collect();
    if files.is_empty() && folders.is_empty() {
        return Ok(None);
    }
    // A marker that lists a path outside the folder is not one ghg wrote.
    let record = record
        .filter(lists_only_inside)
        .ok_or(OutputErrorKind::NotASite)?;
    match (&record, target) {
        (Record::Versions { .. }, BuildTarget::Site) => Err(OutputErrorKind::HoldsVersions),
        (Record::Site(_), BuildTarget::Version(_)) => Err(OutputErrorKind::HoldsSite),
        _ => Ok(()),
    }?;
    let written = written(&record);
    let made = made(&record, &written);
    let stray = folders
        .difference(&made)
        .chain(files.difference(&written))
        .min_by(|one, other| folder_by_folder(one, other));
    if let Some(found) = stray {
        return Err(OutputErrorKind::NotSafe {
            found: found.clone(),
        });
    }
    if let Record::Versions { versions, .. } = &record
        && let Some(name) = versions.keys().find(|name| is_missing(name, &folders))
    {
        return Err(OutputErrorKind::MissingVersion { name: name.clone() });
    }
    already_built(&record, target, moment)?;
    Ok(Some(record))
}

/// Whether every path `record` lists is a path inside the output folder:
/// names joined by `/`, none of them empty, `.` or `..`.
fn lists_only_inside(record: &Record) -> bool {
    match record {
        Record::Site(files) => files.iter().all(is_inside),
        Record::Versions {
            page,
            versions,
            data,
        } => page
            .iter()
            .chain(versions.values().flatten())
            .chain(data)
            .all(is_inside),
    }
}

fn is_inside(path: &FilePath) -> bool {
    path.split('/').all(is_plain_name)
}

fn is_plain_name(name: &str) -> bool {
    !matches!(name, "" | "." | "..")
}

/// `record`, its versions page listing only the files found, and a site
/// only the files whose name at the top of the folder, `tops`, was found
/// there, so that what is removed before the page or the site is written
/// again is always something in the folder, whatever a system reads as a
/// separator.
fn found_only(record: Record, files: &BTreeSet<FilePath>, tops: &BTreeSet<&str>) -> Record {
    match record {
        Record::Versions {
            page,
            versions,
            data,
        } => Record::Versions {
            page: page.intersection(files).cloned().collect(),
            versions,
            data: data.intersection(files).cloned().collect(),
        },
        Record::Site(listed) => Record::Site(
            listed
                .into_iter()
                .filter(|file| is_under(file, tops))
                .collect(),
        ),
    }
}

/// Whether the name at the top of the folder that `file` is, or is in, is one of `tops`.
fn is_under(file: &FilePath, tops: &BTreeSet<&str>) -> bool {
    top_of(file).is_some_and(|top| tops.contains(top))
}

/// The name at the top of the output folder that `path` is, or is in.
fn top_of(path: &FilePath) -> Option<&str> {
    path.split('/').next()
}

/// Paths compared folder by folder, as every list of paths is sorted.
fn folder_by_folder(one: &str, other: &str) -> Ordering {
    one.split('/').cmp(other.split('/'))
}

fn is_missing(name: &VersionName, folders: &BTreeSet<FilePath>) -> bool {
    !folders.contains(name.as_str())
}

/// Every file `record` says ghg wrote, its marker included.
fn written(record: &Record) -> BTreeSet<FilePath> {
    match record {
        Record::Site(files) => files
            .iter()
            .cloned()
            .chain([Record::SITE_MARKER.to_string()])
            .collect(),
        Record::Versions {
            page,
            versions,
            data,
        } => page
            .iter()
            .chain(data)
            .cloned()
            .chain([Record::VERSIONS_MARKER.to_string()])
            .chain(
                versions
                    .iter()
                    .flat_map(|(name, files)| in_folder(name, files)),
            )
            .collect(),
    }
}

/// The paths of `files`, in the folder of the version `name`.
fn in_folder<'a>(
    name: &'a VersionName,
    files: &'a BTreeSet<FilePath>,
) -> impl Iterator<Item = FilePath> + 'a {
    files.iter().map(move |file| format!("{name}/{file}"))
}

/// Every folder ghg made: those that hold the files it wrote, and every version's own.
fn made(record: &Record, written: &BTreeSet<FilePath>) -> BTreeSet<FilePath> {
    let versions = match record {
        Record::Site(_) => None,
        Record::Versions { versions, .. } => Some(versions.keys().map(VersionName::to_string)),
    };
    written
        .iter()
        .flat_map(|file| folders_of(file))
        .chain(versions.into_iter().flatten())
        .collect()
}

/// The folders that hold `file`, from the outermost: `a` and `a/b` for `a/b/c.html`.
fn folders_of(file: &str) -> impl Iterator<Item = FilePath> + '_ {
    file.match_indices('/')
        .filter_map(|(end, _)| file.get(..end))
        .map(str::to_string)
}

/// What stops `target` replacing what is already built, if anything.
fn already_built(
    record: &Record,
    target: &BuildTarget,
    moment: Moment<'_>,
) -> Result<(), OutputErrorKind> {
    let keep = moment == Moment::Checking { replace: false };
    match (record, target) {
        (Record::Versions { versions, .. }, BuildTarget::Version(name)) => {
            if versions.keys().any(|listed| other_capitals(listed, name)) {
                Err(OutputErrorKind::OtherCapitals)
            } else if keep && versions.contains_key(name) {
                Err(OutputErrorKind::AlreadyBuilt)
            } else {
                Ok(())
            }
        }
        _ if keep => Err(OutputErrorKind::AlreadyBuilt),
        _ => Ok(()),
    }
}

/// Whether `listed` and `name` differ only in their capitals, which a file
/// system that does not tell capitals apart would put in the same folder.
fn other_capitals(listed: &VersionName, name: &VersionName) -> bool {
    listed != name && listed.as_str().eq_ignore_ascii_case(name.as_str())
}

#[cfg(test)]
mod tests;
