//! Policy: whether a build may write into an output folder, from what the folder holds, what its marker records, and when it is looked at.

use std::collections::BTreeSet;

use crate::entities::{BuildTarget, FilePath, OutputContents, Record, VersionName};
use crate::errors::OutputErrorKind;

/// When a folder is looked at: when it is checked, before the input is read, or again just
/// before it is written, once the input has been read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Moment {
    /// `replace` says whether a build already there may be replaced.
    Checking {
        replace: bool,
    },
    Writing,
}

/// What the output folder's marker records, `None` when the folder is
/// missing or empty, once nothing stops `target` being written into it; or
/// what stops it. A folder ghg marked must hold only what it wrote there.
pub(crate) fn output_safety(
    contents: OutputContents,
    target: &BuildTarget,
    moment: Moment,
) -> Result<Option<Record>, OutputErrorKind> {
    let (files, folders, record) = match contents {
        OutputContents::Missing => return Ok(None),
        OutputContents::NotAFolder => return Err(OutputErrorKind::NotASite),
        OutputContents::Folder {
            files,
            folders,
            record,
        } => (files, folders, record),
    };
    if files.is_empty() && folders.is_empty() {
        return Ok(None);
    }
    let record = record.ok_or(OutputErrorKind::NotASite)?;
    match (&record, target) {
        (Record::Versions { .. }, BuildTarget::Site) => Err(OutputErrorKind::HoldsVersions),
        (Record::Site(_), BuildTarget::Version(_)) => Err(OutputErrorKind::HoldsSite),
        _ => Ok(()),
    }?;
    let written = written(&record);
    let made = made(&record, &written);
    let stray = folders
        .iter()
        .filter(|folder| !made.contains(*folder))
        .chain(files.iter().filter(|file| !written.contains(*file)))
        .min();
    if let Some(found) = stray {
        return Err(OutputErrorKind::NotSafe {
            found: found.clone(),
        });
    }
    if let Record::Versions { versions, .. } = &record
        && let Some(name) = versions
            .keys()
            .find(|name| !folders.contains(name.as_str()))
    {
        return Err(OutputErrorKind::MissingVersion { name: name.clone() });
    }
    already_built(&record, target, moment)?;
    Ok(Some(record))
}

/// Every file `record` says ghg wrote, its marker included.
fn written(record: &Record) -> BTreeSet<FilePath> {
    match record {
        Record::Site(files) => files
            .iter()
            .cloned()
            .chain([Record::SITE_MARKER.to_string()])
            .collect(),
        Record::Versions { page, versions } => page
            .iter()
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
    moment: Moment,
) -> Result<(), OutputErrorKind> {
    let keep = moment == Moment::Checking { replace: false };
    match (record, target) {
        (Record::Versions { versions, .. }, BuildTarget::Version(name)) => {
            if versions
                .keys()
                .any(|listed| listed != name && listed.as_str().eq_ignore_ascii_case(name.as_str()))
            {
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

#[cfg(test)]
mod tests;
