//! Policy: which files and folders of the input `--only` lets a build read, from the paths it lists.

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use crate::entities::EntryKind;

/// The listed paths inside the input folder, each relative to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ListedSelection {
    files: BTreeSet<PathBuf>,
}

impl ListedSelection {
    /// The paths of `listed` inside `input`, both absolute, each with its `.` parts dropped and
    /// its `..` parts taking away the part before them.
    pub(crate) fn new(input: &Path, listed: impl IntoIterator<Item = PathBuf>) -> Self {
        let input = lexically_normal(input);
        ListedSelection {
            files: listed
                .into_iter()
                .filter_map(|path| relative_to(&input, &path))
                .collect(),
        }
    }

    /// Whether the entry at `relative`, the path below the input folder, is left out: a file
    /// that is not listed, or a folder that holds no listed path at any depth below it.
    pub(crate) fn leaves_out(&self, relative: &Path, kind: EntryKind) -> bool {
        match kind {
            EntryKind::Folder => !self.files.iter().any(|file| file.starts_with(relative)),
            EntryKind::File | EntryKind::Other => !self.files.contains(relative),
        }
    }
}

/// `path` below `input`, if it is inside it.
fn relative_to(input: &Path, path: &Path) -> Option<PathBuf> {
    let path = lexically_normal(path);
    let relative = path.strip_prefix(input).ok()?;
    (relative != Path::new("")).then(|| relative.to_path_buf())
}

fn lexically_normal(path: &Path) -> PathBuf {
    path.components().fold(PathBuf::new(), with_part)
}

fn with_part(mut path: PathBuf, part: Component<'_>) -> PathBuf {
    match part {
        Component::CurDir => {}
        Component::ParentDir => {
            path.pop();
        }
        Component::Prefix(_) | Component::RootDir | Component::Normal(_) => path.push(part),
    }
    path
}

#[cfg(test)]
mod tests {
    use proptest::prelude::proptest;
    use rstest::rstest;

    use super::*;

    fn selection(listed: &[&str]) -> ListedSelection {
        ListedSelection::new(
            Path::new("/work/features"),
            listed.iter().map(PathBuf::from),
        )
    }

    // Rule 205.
    #[rstest]
    #[case::listed_as_written("/work/features/a.feature")]
    #[case::with_a_dot("/work/./features/a.feature")]
    #[case::with_two_dots("/work/other/../features/a.feature")]
    fn a_listed_file_inside_the_input_is_read(#[case] listed: &str) {
        let selection = selection(&[listed]);
        assert!(!selection.leaves_out(Path::new("a.feature"), EntryKind::File));
        assert!(selection.leaves_out(Path::new("b.feature"), EntryKind::File));
    }

    // Rule 205.
    #[rstest]
    #[case::outside_the_input("/work/src/a.feature")]
    #[case::beside_the_input("/work/features-old/a.feature")]
    #[case::the_input_itself("/work/features")]
    fn a_listed_path_outside_the_input_names_nothing(#[case] listed: &str) {
        assert_eq!(selection(&[listed]), selection(&[]));
    }

    // Rule 206.
    #[test]
    fn only_the_folders_on_the_way_to_a_listed_file_are_looked_into() {
        let selection = selection(&["/work/features/billing/old/a.feature"]);
        assert!(!selection.leaves_out(Path::new("billing"), EntryKind::Folder));
        assert!(!selection.leaves_out(Path::new("billing/old"), EntryKind::Folder));
        assert!(selection.leaves_out(Path::new("accounts"), EntryKind::Folder));
        assert!(selection.leaves_out(Path::new("billing/new"), EntryKind::Folder));
        assert!(selection.leaves_out(Path::new("billing/a.feature"), EntryKind::File));
    }

    proptest! {
        // Rule 205: whatever its `.` and `..` parts, a listed path is kept only as a path below
        // the input folder, with none of those parts left.
        #[test]
        fn no_path_outside_the_input_is_kept(parts in proptest::collection::vec("\\.|\\.\\.|features|work|a", 0..8)) {
            let path = Path::new("/").join(parts.join("/"));
            let kept = selection(&[path.to_str().unwrap_or_default()]);
            for file in &kept.files {
                assert!(file.components().all(|part| matches!(part, Component::Normal(_))), "{path:?}: {file:?}");
            }
        }
    }
}
