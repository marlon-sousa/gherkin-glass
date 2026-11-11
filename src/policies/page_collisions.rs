//! Policy: the input files whose pages would collide, with each other, with a folder's page, or with a page of the site's own.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::entities::{Problem, ProblemKind, SiteAddress, TAGS_FOLDER};
use crate::policies::{FOLDER_PAGE, SENTENCES_PAGE, page_address};

/// Every problem of the pages of `paths`, files under `input` in path order: those of
/// [`same_folder_pages`], of [`same_file_pages`], then of [`pages_in_tags_folder`].
pub(crate) fn page_collisions(input: &Path, paths: &[PathBuf]) -> Vec<Problem> {
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
}

/// Every folder whose page an earlier folder in path order has, compared without regard to case.
fn same_folder_pages(input: &Path, pages: &[Page<'_>]) -> Vec<Problem> {
    let mut folders: Vec<String> = pages
        .iter()
        .flat_map(|page| enclosing_folders(&page.address))
        .collect();
    let mut seen = BTreeSet::new();
    folders.retain(|folder| seen.insert(folder.clone()));
    let first: BTreeMap<String, &str> = folders
        .iter()
        .rev()
        .map(String::as_str)
        .map(without_case)
        .collect();
    folders
        .iter()
        .filter_map(|folder| folder_clash(input, folder, &first))
        .collect()
}

/// The folders a page is in, outermost first, each as its path from the input folder.
fn enclosing_folders(page: &SiteAddress) -> Vec<String> {
    let folders = page.folders();
    (1..=folders.len())
        .filter_map(|depth| folders.get(..depth))
        .map(|outer| outer.join("/"))
        .collect()
}

fn without_case(folder: &str) -> (String, &str) {
    (folder.to_lowercase(), folder)
}

/// The problem of `folder` when `first`, keyed without regard to case, holds another spelling of it.
fn folder_clash(input: &Path, folder: &str, first: &BTreeMap<String, &str>) -> Option<Problem> {
    let earlier = *first.get(&folder.to_lowercase())?;
    (earlier != folder).then(|| {
        Problem::new(
            input,
            &input.join(folder),
            None,
            ProblemKind::SameFolderPage {
                page: format!("{folder}/{FOLDER_PAGE}"),
                first: earlier.to_string(),
            },
        )
    })
}

/// Every file whose page, compared without regard to case, is a folder's page or the page of
/// step sentences, or one an earlier file in path order already has.
fn same_file_pages(input: &Path, pages: &[Page<'_>]) -> Vec<Problem> {
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

fn file_clash(input: &Path, page: &Page<'_>, first: &BTreeMap<String, &Path>) -> Option<Problem> {
    reserved(&page.address)
        .or_else(|| same_as_earlier(input, page, first))
        .map(|kind| Problem::new(input, page.path, None, kind))
}

/// What is wrong with a page at the place of the page of step sentences or of a folder's page,
/// compared without regard to case.
fn reserved(page: &SiteAddress) -> Option<ProblemKind> {
    if page.to_string().to_lowercase() == SENTENCES_PAGE {
        return Some(ProblemKind::ReservedPage {
            page: SENTENCES_PAGE.to_string(),
        });
    }
    let folders = page.folders();
    (page.file().to_lowercase() == FOLDER_PAGE).then(|| ProblemKind::SameAsFolderPage {
        page: page.to_string(),
        folder: (!folders.is_empty()).then(|| folders.join("/")),
    })
}

/// What is wrong with a page when `first`, keyed without regard to case, holds another file for it.
fn same_as_earlier(
    input: &Path,
    page: &Page<'_>,
    first: &BTreeMap<String, &Path>,
) -> Option<ProblemKind> {
    let (key, path) = page.keyed();
    let earlier = *first.get(&key)?;
    (earlier != path).then(|| ProblemKind::SamePage {
        page: page.address.to_string(),
        first: Problem::shown_path(input, earlier),
    })
}

/// Every file whose page would be in the folder of the tag pages, its name compared without
/// regard to case, as the other pages are.
fn pages_in_tags_folder(input: &Path, pages: &[Page<'_>]) -> Vec<Problem> {
    pages
        .iter()
        .filter_map(|page| in_tags_folder(input, page))
        .collect()
}

fn in_tags_folder(input: &Path, page: &Page<'_>) -> Option<Problem> {
    let folder = *page.address.folders().first()?;
    (folder.to_lowercase() == TAGS_FOLDER).then(|| {
        Problem::new(
            input,
            page.path,
            None,
            ProblemKind::InTagsFolder {
                page: page.address.to_string(),
            },
        )
    })
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::entities::Problems;

    /// The problems of the pages of `paths`, below the input folder `in`, as they are reported.
    fn collisions(paths: &[&str]) -> String {
        let input = Path::new("in");
        let paths: Vec<_> = paths.iter().map(|path| input.join(path)).collect();
        Problems::new(page_collisions(input, &paths)).to_string()
    }

    #[rstest]
    #[case::with_each_other(
        &["a.feature", "a.story", "A"],
        "A: has the same page, A.html, as a.feature\n\
         a.story: has the same page, a.html, as a.feature\n"
    )]
    #[case::with_the_step_sentences(
        &["Steps.feature", "a.feature", "steps.feature", "sub/steps.feature"],
        "Steps.feature: has the same page, steps.html, as the step sentences\n\
         steps.feature: has the same page, steps.html, as the step sentences\n"
    )]
    #[case::in_the_folder_of_the_tag_pages(
        &["Tags/a.feature", "tags.feature", "b/tags/c.feature"],
        "Tags/a.feature: has its page, Tags/a.html, in the folder tags\n"
    )]
    #[case::folders_differing_only_in_case_once(
        &["A/x.feature", "a/y.feature", "a/z.feature", "a/b/w.feature"],
        "a: has the same page, a/index.html, as the folder A\n"
    )]
    fn pages_that_collide_are_reported(#[case] paths: &[&str], #[case] reported: &str) {
        assert_eq!(collisions(paths), reported);
    }

    #[test]
    fn pages_that_do_not_collide_bring_no_problem() {
        assert_eq!(
            collisions(&["a.feature", "sub/a.feature", "b/c.feature"]),
            ""
        );
    }

    #[test]
    fn a_feature_whose_page_is_a_folder_s_is_reported_in_any_case() {
        let input = Path::new("in");
        let paths = [input.join("INDEX.feature"), input.join("a/b/index.story")];
        let kinds: Vec<_> = page_collisions(input, &paths)
            .into_iter()
            .map(|problem| problem.kind)
            .collect();
        assert_eq!(
            kinds,
            [
                ProblemKind::SameAsFolderPage {
                    page: "INDEX.html".into(),
                    folder: None
                },
                ProblemKind::SameAsFolderPage {
                    page: "a/b/index.html".into(),
                    folder: Some("a/b".into())
                },
            ]
        );
    }
}
