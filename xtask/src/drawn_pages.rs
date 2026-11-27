//! Adapter: the pages the script draws from every expected site, in happy-dom, written beside the site in `cargo xtask build` and compared with the committed ones in `cargo xtask check-script`.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs::{DirEntry, read, read_dir};
use std::path::{Path, PathBuf};

use super::{npm, replace_folder, root, target};

/// The folders, from the repository's root, whose folders named `expected` or `expected-…` are
/// expected sites when they hold the marker of an output folder.
const FIXTURE_FOLDERS: [&str; 2] = ["tests/fixtures", "project-site"];

/// The marker every output folder ghg writes holds.
const MARKER: &str = "ghg-site.json";

/// Draws the pages of every expected site into the folder beside it, `drawn` beside `expected`
/// and `drawn-collapsed` beside `expected-collapsed`.
pub(crate) fn write_drawn_pages() -> Result<(), String> {
    expected_sites()?
        .iter()
        .try_for_each(|site| draw(&root().join(site), &root().join(drawn_folder(site))))
}

/// Draws the pages of every expected site below Cargo's target folder, and fails on the first
/// that differs from the committed one.
pub(crate) fn check_drawn_pages() -> Result<(), String> {
    expected_sites()?.iter().try_for_each(|site| {
        let drawn = target().join("xtask").join(drawn_folder(site));
        draw(&root().join(site), &drawn)?;
        let committed = root().join(drawn_folder(site));
        match first_difference(&drawn, &committed)? {
            Some(file) => Err(not_drawn(&committed.join(file), &drawn_folder(site))),
            None => Ok(()),
        }
    })
}

/// Every expected site, from the repository's root, sorted.
fn expected_sites() -> Result<BTreeSet<PathBuf>, String> {
    FIXTURE_FOLDERS
        .iter()
        .map(|folder| expected_sites_below(Path::new(folder)))
        .collect::<Result<Vec<_>, _>>()
        .map(|found| found.into_iter().flatten().collect())
}

/// The expected sites at or below `folder`, from the repository's root.
fn expected_sites_below(folder: &Path) -> Result<BTreeSet<PathBuf>, String> {
    if folder.file_name().is_some_and(is_expected_name)
        && root().join(folder).join(MARKER).is_file()
    {
        return Ok(BTreeSet::from([folder.to_path_buf()]));
    }
    entries(&root().join(folder))?
        .iter()
        .filter(|entry| entry.path().is_dir())
        .map(|entry| expected_sites_below(&folder.join(entry.file_name())))
        .collect::<Result<Vec<_>, _>>()
        .map(|found| found.into_iter().flatten().collect())
}

/// Whether `name` is `expected` or `expected-…`, as an expected site's folder is named.
fn is_expected_name(name: &OsStr) -> bool {
    name == "expected" || name.to_string_lossy().starts_with("expected-")
}

/// The folder of the pages drawn from `site`, beside it, its name's `expected` as `drawn`.
fn drawn_folder(site: &Path) -> PathBuf {
    let name = site.file_name().unwrap_or_default().to_string_lossy();
    site.with_file_name(name.replacen("expected", "drawn", 1))
}

fn draw(site: &Path, drawn: &Path) -> Result<(), String> {
    replace_folder(drawn, None)?;
    npm(&[
        "run",
        "--silent",
        "draw",
        "--",
        &site.display().to_string(),
        &drawn.display().to_string(),
    ])
}

/// The first file, relative to them, that is in only one of the folders `made` and `committed`,
/// or in both with other bytes; `None` when they hold the same files.
fn first_difference(made: &Path, committed: &Path) -> Result<Option<PathBuf>, String> {
    let made_files = files(made, Path::new(""))?;
    let committed_files = files(committed, Path::new(""))?;
    if let Some(file) = made_files.symmetric_difference(&committed_files).next() {
        return Ok(Some(file.clone()));
    }
    made_files
        .iter()
        .map(|file| differs(made, committed, file))
        .find_map(Result::transpose)
        .transpose()
}

/// `file`, when its bytes in `made` and in `committed` differ.
fn differs(made: &Path, committed: &Path, file: &Path) -> Result<Option<PathBuf>, String> {
    let differ = read_file(&made.join(file))? != read_file(&committed.join(file))?;
    Ok(differ.then(|| file.to_path_buf()))
}

/// The error of a drawn page, `file`, that is not what the script draws, in `folder`.
fn not_drawn(file: &Path, folder: &Path) -> String {
    let shown = file.strip_prefix(root()).unwrap_or(file);
    format!(
        "{} is not what the script draws. Run cargo xtask build, and commit {}.",
        shown.display(),
        folder.display()
    )
}

/// The files below `folder`'s `below`, relative to `folder`; none when `folder` does not exist.
fn files(folder: &Path, below: &Path) -> Result<BTreeSet<PathBuf>, String> {
    let listed = match entries(&folder.join(below)) {
        Err(_) if !folder.exists() => return Ok(BTreeSet::new()),
        listed => listed?,
    };
    listed
        .iter()
        .map(|entry| entry_files(folder, below, entry))
        .collect::<Result<Vec<_>, _>>()
        .map(|found| found.into_iter().flatten().collect())
}

/// The file `entry` of `folder`'s `below`, or the files below it when it is a folder, relative
/// to `folder`.
fn entry_files(folder: &Path, below: &Path, entry: &DirEntry) -> Result<BTreeSet<PathBuf>, String> {
    let path = below.join(entry.file_name());
    if entry.path().is_dir() {
        files(folder, &path)
    } else {
        Ok(BTreeSet::from([path]))
    }
}

fn entries(folder: &Path) -> Result<Vec<DirEntry>, String> {
    read_dir(folder)
        .and_then(Iterator::collect)
        .map_err(|error| format!("{}: {error}", folder.display()))
}

fn read_file(path: &Path) -> Result<Vec<u8>, String> {
    read(path).map_err(|error| format!("{}: {error}", path.display()))
}
