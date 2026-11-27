//! Policy: whether a command may write into an output folder, or remove from it, from what the folder holds, what its marker records, what the command leaves alone there, and when it is looked at.

mod kept_clash;

use std::collections::{BTreeMap, BTreeSet};

use crate::entities::{
    BuildTarget, EntryKind, FilePath, KeptInOutput, OutputContents, OutputMarker, VersionName,
};
use crate::errors::OutputErrorKind;
use crate::policies::rendering::{
    PREVIEWS, SITE_DESCRIPTOR, STORE_FORMAT, VERSIONS, folder_by_folder, read_marker,
};

use kept_clash::kept_clash;

/// When a folder is looked at: when it is checked, before the input is read, or again just
/// before it is written, once the input has been read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Moment<'a> {
    /// `replace` says whether a site or a version already there may be replaced.
    Checking { replace: bool },
    /// `site` is every file the command writes, by its path in the output folder.
    Writing { site: &'a BTreeSet<FilePath> },
}

/// What a command does to its target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Change {
    Add,
    Remove,
}

/// What a safe output folder holds: what its marker lists and the files in it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SafeFolder {
    /// Every file the marker lists; none when the folder is missing or holds only what is kept.
    pub(crate) listed: BTreeSet<FilePath>,
    /// Every file in it, but what is kept and the marker.
    pub(crate) found: BTreeSet<FilePath>,
}

/// What the output folder holds, once nothing stops `change` of `target` in it; or what stops
/// it. A folder gherkin-glass 1 marked, or whose store is newer, is refused first; a folder ghg
/// marked must hold only what it wrote there and what `kept` keeps, which must be nothing ghg
/// writes.
pub(crate) fn output_safety(
    contents: OutputContents,
    target: &BuildTarget,
    change: Change,
    moment: Moment<'_>,
    kept: &KeptInOutput,
) -> Result<SafeFolder, OutputErrorKind> {
    let (files, folders, found_kept, marker) = match contents {
        OutputContents::Missing => return Ok(SafeFolder::default()),
        OutputContents::NotAFolder => return Err(OutputErrorKind::NotASite),
        OutputContents::Folder {
            files,
            folders,
            kept,
            marker,
        } => (files, folders, kept, marker),
    };
    let marker = marker.as_deref().and_then(read_marker);
    if marker == Some(OutputMarker::GherkinGlassOne)
        || files.contains(OutputMarker::GHERKIN_GLASS_ONE_VERSIONS)
    {
        return Err(OutputErrorKind::GherkinGlassOne);
    }
    if let Some(OutputMarker::Store { format, ghg, .. }) = &marker
        && *format > STORE_FORMAT
    {
        return Err(OutputErrorKind::NewerStore { ghg: ghg.clone() });
    }
    let listed = marked(&files, &folders, &found_kept, marker)?;
    let safe = SafeFolder {
        found: files
            .into_iter()
            .filter(|file| file.as_str() != OutputMarker::FILE)
            .collect(),
        listed,
    };
    if !safe.listed.is_empty() {
        holds(&safe, &folders, target, change, moment)?;
    }
    kept_clash(kept, &found_kept, &safe.listed, target, moment)?;
    Ok(safe)
}

/// What the marker lists, none when the folder holds nothing but what is kept, once nothing but
/// what is kept is there that ghg did not write.
fn marked(
    files: &BTreeSet<FilePath>,
    folders: &BTreeSet<FilePath>,
    found_kept: &BTreeMap<FilePath, EntryKind>,
    marker: Option<OutputMarker>,
) -> Result<BTreeSet<FilePath>, OutputErrorKind> {
    // A folder holding what is kept is no stray, though what else it holds may be.
    let holding: BTreeSet<_> = found_kept
        .keys()
        .flat_map(|path| folders_of(path))
        .collect();
    let folders: BTreeSet<_> = folders.difference(&holding).cloned().collect();
    if files.is_empty() && folders.is_empty() {
        return Ok(BTreeSet::new());
    }
    let Some(OutputMarker::Store { files: listed, .. }) = marker else {
        return Err(OutputErrorKind::NotASite);
    };
    let written: BTreeSet<FilePath> = listed
        .iter()
        .cloned()
        .chain([OutputMarker::FILE.to_string()])
        .collect();
    let made: BTreeSet<FilePath> = written.iter().flat_map(|file| folders_of(file)).collect();
    let stray = folders
        .difference(&made)
        .chain(files.difference(&written))
        .min_by(|one, other| folder_by_folder(one, other));
    match stray {
        Some(found) => Err(OutputErrorKind::NotSafe {
            found: found.clone(),
        }),
        None => Ok(listed),
    }
}

/// What stops `change` of `target` in a marked folder for what its store holds: versions
/// beside a site without versions, a version whose folder is missing, a name that differs from
/// one held only in its capitals, or a site or a version already built that is not replaced.
fn holds(
    safe: &SafeFolder,
    folders: &BTreeSet<FilePath>,
    target: &BuildTarget,
    change: Change,
    moment: Moment<'_>,
) -> Result<(), OutputErrorKind> {
    let held: BTreeSet<&str> = safe
        .listed
        .intersection(&safe.found)
        .map(String::as_str)
        .collect();
    let versions: Vec<&str> = named_in(&held, VERSIONS).collect();
    let previews: Vec<&str> = named_in(&held, PREVIEWS).collect();
    let site = held.contains(SITE_DESCRIPTOR);
    if change == Change::Add {
        match target {
            BuildTarget::Site if !versions.is_empty() => Err(OutputErrorKind::HoldsVersions),
            BuildTarget::Version(_) if site => Err(OutputErrorKind::HoldsSite),
            _ => Ok(()),
        }?;
    }
    if let Some(name) = versions.iter().find(|name| !folders.contains(**name)) {
        return Err(OutputErrorKind::MissingVersion {
            name: (*name).to_string(),
        });
    }
    if change == Change::Remove {
        return Ok(());
    }
    let others = match target {
        BuildTarget::Site => Vec::new(),
        BuildTarget::Version(_) => versions,
        BuildTarget::Preview(_) => previews,
    };
    let name = target_name(target);
    if others.iter().any(|other| other_capitals(other, name)) {
        return Err(OutputErrorKind::OtherCapitals);
    }
    let keep = moment == Moment::Checking { replace: false };
    let built = match target {
        BuildTarget::Site => site,
        BuildTarget::Version(_) => others.contains(&name),
        BuildTarget::Preview(_) => false,
    };
    if keep && built {
        return Err(OutputErrorKind::AlreadyBuilt);
    }
    Ok(())
}

/// The names of the descriptors among `held` in the folder `folder` of the store.
fn named_in<'a>(held: &BTreeSet<&'a str>, folder: &'a str) -> impl Iterator<Item = &'a str> {
    held.iter()
        .filter_map(move |file| file.strip_prefix(folder)?.strip_suffix(".json"))
        .filter(|name| VersionName::new(name).is_some())
}

fn target_name(target: &BuildTarget) -> &str {
    match target {
        BuildTarget::Site => "",
        BuildTarget::Version(name) | BuildTarget::Preview(name) => name.as_str(),
    }
}

/// Whether `listed` and `name` differ only in their capitals, which a file system that does not
/// tell capitals apart would put in the same folder.
fn other_capitals(listed: &str, name: &str) -> bool {
    listed != name && listed.eq_ignore_ascii_case(name)
}

/// The folders that hold `file`, from the outermost: `a` and `a/b` for `a/b/c.html`.
fn folders_of(file: &str) -> impl Iterator<Item = FilePath> + '_ {
    file.match_indices('/')
        .filter_map(|(end, _)| file.get(..end))
        .map(str::to_string)
}

#[cfg(test)]
mod tests;
