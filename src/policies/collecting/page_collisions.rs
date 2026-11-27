//! Policy: the input files whose pages would collide, with each other or with a page of the site's own, their names compared exactly.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::entities::{PageClash, TAGS_FOLDER};
use crate::errors::{InputError, InputErrorKind};
use crate::policies::page_name;

/// The page of step sentences.
const SENTENCES_PAGE: &str = "steps";

/// The page of search results.
const SEARCH_PAGE: &str = "search";

/// Every clash of the pages of `paths`, files under `input` in path order: of a page with a
/// page of the site's own or one an earlier file has, then of a page in the folder of the tag
/// pages.
pub(crate) fn page_collisions(input: &Path, paths: &[PathBuf]) -> Vec<PageClash> {
    let pages: Vec<_> = paths.iter().map(|path| Page::of(input, path)).collect();
    [
        same_file_pages(input, &pages),
        pages_in_tags_folder(input, &pages),
    ]
    .concat()
}

/// An input file and the name of its page.
struct Page<'a> {
    path: &'a Path,
    name: String,
}

impl<'a> Page<'a> {
    fn of(input: &Path, path: &'a Path) -> Self {
        Page {
            path,
            name: page_name(path.strip_prefix(input).unwrap_or(path)),
        }
    }

    fn clash(&self, input: &Path, kind: InputErrorKind) -> PageClash {
        PageClash {
            path: self.path.to_path_buf(),
            error: InputError::new(input, self.path, None, kind),
        }
    }
}

/// Every file whose page is the page of step sentences or the page of search results, or one
/// an earlier file in path order already has.
fn same_file_pages(input: &Path, pages: &[Page<'_>]) -> Vec<PageClash> {
    let first: BTreeMap<&str, &Path> = pages
        .iter()
        .rev()
        .filter(|page| reserved(&page.name).is_none())
        .map(|page| (page.name.as_str(), page.path))
        .collect();
    pages
        .iter()
        .filter_map(|page| file_clash(input, page, &first))
        .collect()
}

fn file_clash(input: &Path, page: &Page<'_>, first: &BTreeMap<&str, &Path>) -> Option<PageClash> {
    reserved(&page.name)
        .or_else(|| same_as_earlier(input, page, first))
        .map(|kind| page.clash(input, kind))
}

/// What is wrong with a page named as the page of step sentences or of search results.
fn reserved(name: &str) -> Option<InputErrorKind> {
    match name {
        SENTENCES_PAGE => Some(InputErrorKind::ReservedPage {
            page: SENTENCES_PAGE.to_string(),
        }),
        SEARCH_PAGE => Some(InputErrorKind::SearchPageClash {
            page: SEARCH_PAGE.to_string(),
        }),
        _ => None,
    }
}

/// What is wrong with a page when `first` holds another file for it.
fn same_as_earlier(
    input: &Path,
    page: &Page<'_>,
    first: &BTreeMap<&str, &Path>,
) -> Option<InputErrorKind> {
    let earlier = *first.get(page.name.as_str())?;
    (earlier != page.path).then(|| InputErrorKind::SamePage {
        page: page.name.clone(),
        first: InputError::shown_path(input, earlier),
    })
}

/// Every file whose page would be in the folder of the tag pages.
fn pages_in_tags_folder(input: &Path, pages: &[Page<'_>]) -> Vec<PageClash> {
    pages
        .iter()
        .filter(|page| page.name.split('/').count() > 1)
        .filter(|page| page.name.split('/').next() == Some(TAGS_FOLDER))
        .map(|page| {
            let kind = InputErrorKind::InTagsFolder {
                page: page.name.clone(),
            };
            page.clash(input, kind)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::errors::InputErrors;

    /// The errors of the pages of `paths`, below the input folder `in`, in the order they are
    /// reported, each as the path it is said of and what is wrong.
    fn collisions(paths: &[&str]) -> Vec<(String, InputErrorKind)> {
        let input = Path::new("in");
        let paths: Vec<_> = paths.iter().map(|path| input.join(path)).collect();
        let errors: Vec<_> = page_collisions(input, &paths)
            .into_iter()
            .map(|clash| clash.error)
            .collect();
        InputErrors::new(errors)
            .as_slice()
            .iter()
            .map(|error| (error.path.clone(), error.kind.clone()))
            .collect()
    }

    fn same_page(path: &str, page: &str, first: &str) -> (String, InputErrorKind) {
        let kind = InputErrorKind::SamePage {
            page: page.into(),
            first: first.into(),
        };
        (path.into(), kind)
    }

    fn reserved(path: &str) -> (String, InputErrorKind) {
        let page = "steps".into();
        (path.into(), InputErrorKind::ReservedPage { page })
    }

    fn search_page(path: &str) -> (String, InputErrorKind) {
        let page = "search".into();
        (path.into(), InputErrorKind::SearchPageClash { page })
    }

    #[rstest]
    // Rule 37: names compared exactly, so `A` is a page of its own.
    #[case::with_each_other(
        &["a.feature", "a.story", "A"],
        vec![same_page("a.story", "a", "a.feature")]
    )]
    // Rule 142.
    #[case::with_the_step_sentences(
        &["Steps.feature", "a.feature", "steps.feature", "sub/steps.feature"],
        vec![reserved("steps.feature")]
    )]
    // Rule 215.
    #[case::with_the_search_results(
        &["Search.feature", "a.feature", "search.story", "sub/search.feature"],
        vec![search_page("search.story")]
    )]
    // Rule 101.
    #[case::in_the_folder_of_the_tag_pages(
        &["Tags/a.feature", "tags.feature", "tags/c.feature", "b/tags/c.feature"],
        vec![("tags/c.feature".into(), InputErrorKind::InTagsFolder { page: "tags/c".into() })]
    )]
    fn pages_that_collide_are_reported(
        #[case] paths: &[&str],
        #[case] reported: Vec<(String, InputErrorKind)>,
    ) {
        assert_eq!(collisions(paths), reported);
    }

    #[test]
    fn pages_that_do_not_collide_bring_no_error() {
        assert_eq!(
            collisions(&[
                "a.feature",
                "sub/a.feature",
                "b/c.feature",
                "index.feature",
                "A/x.feature",
                "a/y.feature"
            ]),
            []
        );
    }
}
