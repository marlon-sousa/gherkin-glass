//! Policy: the input files whose pages would collide, with each other, with a folder's page, or with a page of the site's own.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::entities::{PageClash, SiteAddress, TAGS_FOLDER};
use crate::errors::{InputError, InputErrorKind};
use crate::policies::{FOLDER_PAGE, SENTENCES_PAGE, page_address};

/// Every clash of the pages of `paths`, files under `input` in path order: those of
/// [`same_folder_pages`], of [`same_file_pages`], then of [`pages_in_tags_folder`].
pub(crate) fn page_collisions(input: &Path, paths: &[PathBuf]) -> Vec<PageClash> {
    let pages: Vec<_> = paths.iter().map(|path| Page::of(input, path)).collect();
    [
        same_folder_pages(input, &pages),
        same_file_pages(input, &pages),
        pages_in_tags_folder(input, &pages),
    ]
    .concat()
}

/// An input file and the address of its page.
struct Page<'a> {
    path: &'a Path,
    address: SiteAddress,
}

impl<'a> Page<'a> {
    fn of(input: &Path, path: &'a Path) -> Self {
        Page {
            path,
            address: page_address(path.strip_prefix(input).unwrap_or(path)),
        }
    }

    /// Its address without regard to case, since some file systems do not regard it, and its file.
    fn keyed(&self) -> (String, &'a Path) {
        (self.address.to_string().to_lowercase(), self.path)
    }

    /// The folders it is in below the input folder, outermost first, each as its path from the
    /// input folder in the site, with `/` between folders, and as its path in the input.
    fn enclosing_folders(&self) -> Vec<(String, PathBuf)> {
        let folders = self.address.folders();
        let mut paths: Vec<_> = self
            .path
            .ancestors()
            .skip(1)
            .take(folders.len())
            .map(Path::to_path_buf)
            .collect();
        paths.reverse();
        (1..=folders.len())
            .filter_map(|depth| folders.get(..depth))
            .map(|outer| outer.join("/"))
            .zip(paths)
            .collect()
    }

    fn clash(&self, input: &Path, kind: InputErrorKind) -> PageClash {
        PageClash {
            path: self.path.to_path_buf(),
            error: InputError::new(input, self.path, None, kind),
        }
    }
}

/// Every folder whose page an earlier folder in path order has, compared without regard to case.
fn same_folder_pages(input: &Path, pages: &[Page<'_>]) -> Vec<PageClash> {
    let mut folders: Vec<(String, PathBuf)> =
        pages.iter().flat_map(Page::enclosing_folders).collect();
    let mut seen = BTreeSet::new();
    folders.retain(|(folder, _)| seen.insert(folder.clone()));
    let first: BTreeMap<String, &str> = folders.iter().rev().map(without_case).collect();
    folders
        .iter()
        .filter_map(|folder| folder_clash(input, folder, &first))
        .collect()
}

fn without_case((folder, _): &(String, PathBuf)) -> (String, &str) {
    (folder.to_lowercase(), folder)
}

/// The clash of `folder` when `first`, keyed without regard to case, holds another spelling of it.
fn folder_clash(
    input: &Path,
    (folder, path): &(String, PathBuf),
    first: &BTreeMap<String, &str>,
) -> Option<PageClash> {
    let earlier = *first.get(&folder.to_lowercase())?;
    (earlier != folder).then(|| PageClash {
        path: path.clone(),
        error: InputError::new(
            input,
            path,
            None,
            InputErrorKind::SameFolderPage {
                page: format!("{folder}/{FOLDER_PAGE}"),
                first: earlier.to_string(),
            },
        ),
    })
}

/// Every file whose page, compared without regard to case, is a folder's page or the page of
/// step sentences, or one an earlier file in path order already has.
fn same_file_pages(input: &Path, pages: &[Page<'_>]) -> Vec<PageClash> {
    let first: BTreeMap<String, &Path> = pages
        .iter()
        .rev()
        .filter(is_a_feature_s_own)
        .map(Page::keyed)
        .collect();
    pages
        .iter()
        .filter_map(|page| file_clash(input, page, &first))
        .collect()
}

fn is_a_feature_s_own(page: &&Page<'_>) -> bool {
    reserved(&page.address).is_none()
}

fn file_clash(input: &Path, page: &Page<'_>, first: &BTreeMap<String, &Path>) -> Option<PageClash> {
    reserved(&page.address)
        .or_else(|| same_as_earlier(input, page, first))
        .map(|kind| page.clash(input, kind))
}

/// What is wrong with a page at the place of the page of step sentences or of a folder's page,
/// compared without regard to case.
fn reserved(page: &SiteAddress) -> Option<InputErrorKind> {
    if page.to_string().to_lowercase() == SENTENCES_PAGE {
        return Some(InputErrorKind::ReservedPage {
            page: SENTENCES_PAGE.to_string(),
        });
    }
    let folders = page.folders();
    (page.file().to_lowercase() == FOLDER_PAGE).then(|| InputErrorKind::SameAsFolderPage {
        page: page.to_string(),
        folder: (!folders.is_empty()).then(|| folders.join("/")),
    })
}

/// What is wrong with a page when `first`, keyed without regard to case, holds another file for it.
fn same_as_earlier(
    input: &Path,
    page: &Page<'_>,
    first: &BTreeMap<String, &Path>,
) -> Option<InputErrorKind> {
    let (key, path) = page.keyed();
    let earlier = *first.get(&key)?;
    (earlier != path).then(|| InputErrorKind::SamePage {
        page: page.address.to_string(),
        first: InputError::shown_path(input, earlier),
    })
}

/// Every file whose page would be in the folder of the tag pages, its name compared without
/// regard to case, as the other pages are.
fn pages_in_tags_folder(input: &Path, pages: &[Page<'_>]) -> Vec<PageClash> {
    pages
        .iter()
        .filter_map(|page| in_tags_folder(input, page))
        .collect()
}

fn in_tags_folder(input: &Path, page: &Page<'_>) -> Option<PageClash> {
    let folder = *page.address.folders().first()?;
    (folder.to_lowercase() == TAGS_FOLDER).then(|| {
        let page_address = page.address.to_string();
        page.clash(input, InputErrorKind::InTagsFolder { page: page_address })
    })
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
        let page = "steps.html".into();
        (path.into(), InputErrorKind::ReservedPage { page })
    }

    #[rstest]
    #[case::with_each_other(
        &["a.feature", "a.story", "A"],
        vec![same_page("A", "A.html", "a.feature"), same_page("a.story", "a.html", "a.feature")]
    )]
    #[case::with_the_step_sentences(
        &["Steps.feature", "a.feature", "steps.feature", "sub/steps.feature"],
        vec![reserved("Steps.feature"), reserved("steps.feature")]
    )]
    #[case::in_the_folder_of_the_tag_pages(
        &["Tags/a.feature", "tags.feature", "b/tags/c.feature"],
        vec![("Tags/a.feature".into(), InputErrorKind::InTagsFolder { page: "Tags/a.html".into() })]
    )]
    #[case::folders_differing_only_in_case_once(
        &["A/x.feature", "a/y.feature", "a/z.feature", "a/b/w.feature"],
        vec![(
            "a".into(),
            InputErrorKind::SameFolderPage { page: "a/index.html".into(), first: "A".into() }
        )]
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
            collisions(&["a.feature", "sub/a.feature", "b/c.feature"]),
            []
        );
    }

    #[test]
    fn a_feature_whose_page_is_a_folder_s_is_reported_in_any_case() {
        let input = Path::new("in");
        let paths = [input.join("INDEX.feature"), input.join("a/b/index.story")];
        let kinds: Vec<_> = page_collisions(input, &paths)
            .into_iter()
            .map(|clash| clash.error.kind)
            .collect();
        assert_eq!(
            kinds,
            [
                InputErrorKind::SameAsFolderPage {
                    page: "INDEX.html".into(),
                    folder: None
                },
                InputErrorKind::SameAsFolderPage {
                    page: "a/b/index.html".into(),
                    folder: Some("a/b".into())
                },
            ]
        );
    }

    #[test]
    fn each_clash_names_the_path_of_its_file_or_folder_in_the_input() {
        let input = Path::new("in");
        let paths = [
            input.join("A/b/x.feature"),
            input.join("a/y.feature"),
            input.join("tags.feature"),
            input.join("Tags/c.feature"),
        ];
        let found: Vec<_> = page_collisions(input, &paths)
            .into_iter()
            .map(|clash| clash.path)
            .collect();
        assert_eq!(found, [input.join("a"), input.join("Tags/c.feature")]);
    }
}
