//! Policy: whether a burst of changes under `--watch` calls for a rebuild.

use std::path::{Path, PathBuf};

use crate::entities::Burst;
use crate::policies::IGNORE_FILE;

/// What a build reads: the configuration file, and the files the
/// `.ghg-ignore` rules select from the input, before the burst and after it.
pub(crate) struct Read<'a> {
    pub config: &'a Path,
    pub before: &'a [PathBuf],
    pub after: &'a [PathBuf],
}

/// A burst calls for a rebuild when it may have changed what a build reads:
/// the configuration file, a `.ghg-ignore` file, a file the rules select,
/// whether it was there before the burst or only after it, or a folder
/// whose selected files differ before and after, such as one moved in or
/// removed. A folder whose selected files are the same, which some systems
/// report when a file inside it is made, does not. A burst whose changes
/// were missed always does.
pub(crate) fn calls_for_rebuild(burst: &Burst, read: &Read) -> bool {
    match burst {
        Burst::Changed(paths) => paths.iter().any(|path| matters(path, read)),
        Burst::Missed => true,
    }
}

/// The files to compare the next burst with. A file the site served was
/// built from is forgotten only by a rebuild, since a burst still to come
/// may be the one that removed it.
pub(crate) fn remembered(before: &[PathBuf], after: Vec<PathBuf>, rebuilt: bool) -> Vec<PathBuf> {
    if rebuilt {
        return after;
    }
    let mut files: Vec<PathBuf> = before.iter().cloned().chain(after).collect();
    files.sort();
    files.dedup();
    files
}

fn matters(path: &Path, read: &Read) -> bool {
    path == read.config
        || path.file_name() == Some(IGNORE_FILE.as_ref())
        || read
            .before
            .iter()
            .chain(read.after)
            .any(|file| file == path)
        || under(path, read.before).ne(under(path, read.after))
}

/// The files of `files` inside the folder `folder`.
fn under<'a>(folder: &'a Path, files: &'a [PathBuf]) -> impl Iterator<Item = &'a PathBuf> {
    files.iter().filter(move |file| file.starts_with(folder))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(paths: &[&str]) -> Vec<PathBuf> {
        paths.iter().map(PathBuf::from).collect()
    }

    #[test]
    fn a_change_rebuilds_only_when_a_build_reads_it() {
        let before = paths(&["features/gone.feature", "features/sub/kept.feature"]);
        let after = paths(&[
            "features/moved/in.feature",
            "features/new.feature",
            "features/sub/kept.feature",
        ]);
        let read = Read {
            config: Path::new("ghg.toml"),
            before: &before,
            after: &after,
        };
        let cases = [
            ("ghg.toml", true),
            ("features/sub/.ghg-ignore", true),
            ("features/gone.feature", true),
            ("features/new.feature", true),
            ("features/moved", true),
            ("features", true),
            ("features/sub", false),
            ("features/notes.txt", false),
            ("features/.kept.feature.swp", false),
            ("ghg.toml~", false),
            ("features/empty", false),
        ];
        for (changed, rebuilds) in cases {
            let burst = Burst::Changed(paths(&["features/notes.txt~", changed]));
            assert_eq!(calls_for_rebuild(&burst, &read), rebuilds, "{changed}");
        }
    }

    #[test]
    fn a_file_removed_is_remembered_until_a_rebuild() {
        let before = paths(&["features/a.feature", "features/x.feature"]);
        let after = paths(&["features/a.feature", "features/new.feature"]);
        assert_eq!(
            remembered(&before, after.clone(), false),
            paths(&[
                "features/a.feature",
                "features/new.feature",
                "features/x.feature"
            ])
        );
        assert_eq!(remembered(&before, after.clone(), true), after);
        // So a later burst that names the removed file still rebuilds.
        let read = Read {
            config: Path::new("ghg.toml"),
            before: &remembered(&before, Vec::new(), false),
            after: &[],
        };
        let burst = Burst::Changed(paths(&["features/x.feature"]));
        assert!(calls_for_rebuild(&burst, &read));
    }

    #[test]
    fn a_burst_whose_changes_were_missed_rebuilds() {
        let read = Read {
            config: Path::new("ghg.toml"),
            before: &[],
            after: &[],
        };
        assert!(calls_for_rebuild(&Burst::Missed, &read));
    }
}
