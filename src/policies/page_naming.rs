//! Policy: the name of a feature file's page, which the input's checks and the site's store both name.

use std::path::{Component, Path};

/// The name of the page of the feature file at `path`, relative to the input folder, which
/// follows the `#` of its address: its parts joined by `/`, without its last extension.
pub(crate) fn page_name(path: &Path) -> String {
    let mut parts = names_of(path);
    if let Some(name) = parts.last_mut() {
        *name = without_extension(name).to_string();
    }
    parts.join("/")
}

fn names_of(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|part| match part {
            Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect()
}

fn without_extension(name: &str) -> &str {
    match name.rfind('.') {
        // A name that starts with its only dot, such as `.feature`, has no extension.
        Some(dot) if dot > 0 => &name[..dot],
        _ => name,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rstest::rstest;

    use super::*;

    // Rule 36: a feature file's page is its path without its last extension.
    #[rstest]
    #[case::at_the_root(&["refunds.feature"], "refunds")]
    #[case::in_folders(&["billing", "archive", "closed #1.feature"], "billing/archive/closed #1")]
    #[case::only_the_last_extension(&["a.b.feature"], "a.b")]
    #[case::a_name_without_an_extension(&["README"], "README")]
    #[case::a_leading_dot_is_no_extension(&[".feature"], ".feature")]
    fn a_feature_file_s_page_is_named_by_its_path_without_its_extension(
        #[case] path: &[&str],
        #[case] name: &str,
    ) {
        assert_eq!(page_name(&PathBuf::from_iter(path)), name);
    }
}
