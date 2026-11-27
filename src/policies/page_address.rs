//! Policy: the addresses of the pages that more than one part of the code names, relative to the site's root: a feature file's page, and its name in the store and a folder's page, which the input's checks name too, the step sentences, the search results, and the tag index, which the tags' names leave free.

use std::path::{Component, Path};

use crate::entities::{SiteAddress, TAGS_FOLDER};

/// A folder's page, in its folder of the site.
pub(crate) const FOLDER_PAGE: &str = "index.html";

/// The page of step sentences, at the site's root.
pub(crate) const SENTENCES_PAGE: &str = "steps.html";

/// The page of search results, at the site's root.
pub(crate) const SEARCH_PAGE: &str = "search.html";

/// The name of the index of every tag, in the folder of the tag pages, which no tag's page takes.
pub(crate) const TAG_INDEX_NAME: &str = "index";

/// The index of every tag, relative to the site's root.
pub(crate) fn tag_index_address() -> SiteAddress {
    SiteAddress::of(&[TAGS_FOLDER], &format!("{TAG_INDEX_NAME}.html"))
}

/// `path` is relative to the input folder. Its parts are joined by `/`, and
/// its last extension is replaced by `html`, or `.html` added when it has none.
pub(crate) fn page_address(path: &Path) -> SiteAddress {
    let parts = names_of(path);
    let Some((name, folders)) = parts.split_last() else {
        return SiteAddress::default();
    };
    SiteAddress::of(folders, &format!("{}.html", without_extension(name)))
}

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

    #[rstest]
    #[case::the_last_extension_becomes_html(&["refunds.feature"], "refunds.html")]
    #[case::folders_are_joined_by_slashes(&["billing", "refunds.feature"], "billing/refunds.html")]
    #[case::any_extension(&["stories", "read.story"], "stories/read.html")]
    #[case::only_the_last_extension(&["a.b.feature"], "a.b.html")]
    #[case::a_name_without_an_extension_gets_one(&["README"], "README.html")]
    #[case::a_leading_dot_is_no_extension(&[".feature"], ".feature.html")]
    fn a_feature_file_s_page_is_its_path_ending_in_html(#[case] path: &[&str], #[case] page: &str) {
        assert_eq!(page_address(&PathBuf::from_iter(path)).as_str(), page);
    }

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
