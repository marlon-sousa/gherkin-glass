//! Policy: whether the paths that changed under `--watch` call for a rebuild.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::entities::IGNORE_FILE;

/// What a build reads: the files the `.ghg-ignore` rules allow in the input, before the changes
/// and after them.
pub(crate) struct Read<'a> {
    pub(crate) before: &'a [PathBuf],
    pub(crate) after: &'a [PathBuf],
}

impl Read<'_> {
    /// Paths that changed call for a rebuild when one of them may have changed what a build
    /// reads: a `.ghg-ignore` file, a file the rules allow, whether it was there before the
    /// changes or only after them, or a folder whose allowed files differ before and after, such
    /// as one moved in or removed. A folder whose allowed files are the same, which some systems
    /// report when a file inside it is made, does not.
    pub(crate) fn calls_for_rebuild(&self, paths: &BTreeSet<PathBuf>) -> bool {
        paths.iter().any(|path| self.matters(path))
    }

    fn matters(&self, path: &Path) -> bool {
        let file = path.to_path_buf();
        path.file_name() == Some(IGNORE_FILE.as_ref())
            || self.before.contains(&file)
            || self.after.contains(&file)
            || under(path, self.before).ne(under(path, self.after))
    }
}

/// The files of `files` inside the folder `folder`.
fn under<'a>(folder: &'a Path, files: &'a [PathBuf]) -> impl Iterator<Item = &'a PathBuf> {
    files.iter().filter(move |file| file.starts_with(folder))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    fn paths(paths: &[&str]) -> Vec<PathBuf> {
        paths.iter().map(PathBuf::from).collect()
    }

    #[rstest]
    // Rule 65.
    #[case::a_ghg_ignore_file("features/sub/.ghg-ignore", true)]
    #[case::a_file_allowed_before("features/gone.feature", true)]
    #[case::a_file_allowed_after("features/new.feature", true)]
    #[case::a_folder_moved_in("features/moved", true)]
    #[case::the_input_folder("features", true)]
    #[case::a_folder_whose_files_are_the_same("features/sub", false)]
    #[case::a_file_left_out("features/notes.txt", false)]
    #[case::an_editor_s_swap_file("features/.kept.feature.swp", false)]
    #[case::an_empty_folder("features/empty", false)]
    fn a_change_calls_for_a_rebuild_only_when_a_build_reads_it(
        #[case] changed: &str,
        #[case] rebuilds: bool,
    ) {
        let before = paths(&["features/gone.feature", "features/sub/kept.feature"]);
        let after = paths(&[
            "features/moved/in.feature",
            "features/new.feature",
            "features/sub/kept.feature",
        ]);
        let read = Read {
            before: &before,
            after: &after,
        };
        let changed: BTreeSet<_> = paths(&["features/notes.txt~", changed])
            .into_iter()
            .collect();
        assert_eq!(read.calls_for_rebuild(&changed), rebuilds);
    }
}
