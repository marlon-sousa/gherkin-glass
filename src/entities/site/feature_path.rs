//! Entity: a feature file's path below the input folder, as the pages show it.

use std::path::{Component, Path};
use std::str::Split;

use derive_more::Display;

/// The names of the folders the file is in, from the outermost, then the file's name, each as
/// written and joined by `/`.
#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub(crate) struct FeaturePath {
    path: String,
}

impl FeaturePath {
    /// The path of the file at `path`, relative to the input folder.
    pub(crate) fn below_input(path: &Path) -> FeaturePath {
        FeaturePath {
            path: path
                .components()
                .filter_map(normal_name)
                .collect::<Vec<_>>()
                .join("/"),
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.path
    }

    pub(crate) fn names(&self) -> Split<'_, char> {
        self.path.split('/')
    }

    /// The file's own name.
    pub(crate) fn file(&self) -> &str {
        self.names().next_back().unwrap_or_default()
    }
}

fn normal_name(part: Component<'_>) -> Option<String> {
    match part {
        Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::at_the_input_s_root(&["a.feature"], "a.feature", &["a.feature"], "a.feature")]
    #[case::two_folders_down(
        &["billing", "refunds", "a b.feature"],
        "billing/refunds/a b.feature",
        &["billing", "refunds", "a b.feature"],
        "a b.feature"
    )]
    #[case::after_the_current_folder(
        &[".", "sub", "a.feature"],
        "sub/a.feature",
        &["sub", "a.feature"],
        "a.feature"
    )]
    fn a_feature_path_is_its_folders_then_its_file_joined_by_slashes(
        #[case] parts: &[&str],
        #[case] written: &str,
        #[case] names: &[&str],
        #[case] file: &str,
    ) {
        let path = FeaturePath::below_input(&parts.iter().collect::<PathBuf>());
        assert_eq!(path.as_str(), written);
        assert_eq!(path.names().collect::<Vec<_>>(), names);
        assert_eq!(path.file(), file);
    }
}
