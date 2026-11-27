//! Adapter: the pages the script draws from every expected site, in happy-dom, written beside the site in `cargo xtask build` and compared with the committed ones in `cargo xtask check-script`.

use std::collections::BTreeSet;
use std::fs::{read, read_dir};
use std::path::{Path, PathBuf};

use super::{npm, replace_folder, root, target};

/// The expected sites, from the repository's root, whose pages are drawn.
const EXPECTED_SITES: [&str; 12] = [
    "tests/fixtures/pages/expected",
    "tests/fixtures/pages/expected-collapsed",
    "tests/fixtures/backgrounds/expected",
    "tests/fixtures/backgrounds/expected-unchecked",
    "tests/fixtures/tags/expected",
    "tests/fixtures/highlighting/expected",
    "tests/fixtures/front-page/expected",
    "tests/fixtures/only/expected",
    "tests/fixtures/only/expected-preview",
    "tests/fixtures/versions/expected",
    "tests/fixtures/versions/expected-removed",
    "project-site/expected",
];

/// Draws the pages of every expected site into the folder beside it, `drawn` beside `expected`
/// and `drawn-collapsed` beside `expected-collapsed`.
pub(crate) fn write_drawn_pages() -> Result<(), String> {
    EXPECTED_SITES
        .iter()
        .try_for_each(|site| draw(&root().join(site), &root().join(drawn_folder(site))))
}

/// Draws the pages of every expected site below Cargo's target folder, and fails on the first
/// that differs from the committed one.
pub(crate) fn check_drawn_pages() -> Result<(), String> {
    EXPECTED_SITES.iter().try_for_each(|site| {
        let drawn = target().join("xtask").join(drawn_folder(site));
        draw(&root().join(site), &drawn)?;
        same_files(&drawn, &root().join(drawn_folder(site))).map_err(|file| {
            format!(
                "{file} is not what the script draws. Run cargo xtask build, and commit {}.",
                drawn_folder(site)
            )
        })
    })
}

/// The folder of the pages drawn from `site`, beside it, its name's `expected` as `drawn`.
fn drawn_folder(site: &str) -> String {
    let (parent, name) = site.rsplit_once('/').unwrap_or(("", site));
    format!("{parent}/{}", name.replacen("expected", "drawn", 1))
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

/// Whether the folders `made` and `committed` hold the same files, the same bytes in each; the
/// error is the first file, from the repository's root, that differs or is in one only.
fn same_files(made: &Path, committed: &Path) -> Result<(), String> {
    let made_files = files(made, Path::new(""));
    let committed_files = files(committed, Path::new(""));
    let differing = made_files
        .symmetric_difference(&committed_files)
        .chain(
            made_files
                .iter()
                .filter(|file| read(made.join(file)).ok() != read(committed.join(file)).ok()),
        )
        .next();
    differing.map_or(Ok(()), |file| {
        Err(committed
            .join(file)
            .strip_prefix(root())
            .unwrap_or(file)
            .display()
            .to_string())
    })
}

/// The files below `folder`'s `below`, relative to `folder`; none when it does not exist.
fn files(folder: &Path, below: &Path) -> BTreeSet<PathBuf> {
    read_dir(folder.join(below))
        .into_iter()
        .flatten()
        .flatten()
        .flat_map(|entry| {
            let path = below.join(entry.file_name());
            if entry.path().is_dir() {
                files(folder, &path)
            } else {
                BTreeSet::from([path])
            }
        })
        .collect()
}
