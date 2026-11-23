//! Handler: what `ghg build` says of a build that nothing stopped, each note with the kind a program reads and the sentence a person reads.

use std::path::Path;

use i18n_embed_fl::fl;
use serde::Serialize;

use crate::entities::{BuildReport, BuiltSite, NothingToBuild};
use crate::policies::CATALOGUE;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct BuildNote {
    kind: NoteKind,
    pub(super) message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum NoteKind {
    NoTitle,
    LargeSearchDocuments,
    NothingToBuild,
}

/// The notes of `report`, in the order they are printed: for a site written, the hint of a site
/// with no title given, then the warning of large search documents, either left out when it
/// does not apply; for nothing to build, why nothing was written.
pub(super) fn notes_of(report: &BuildReport) -> Vec<BuildNote> {
    match report {
        BuildReport::Written { site, .. } => site_notes(site),
        BuildReport::NothingToBuild(nothing) => vec![BuildNote {
            kind: NoteKind::NothingToBuild,
            message: nothing_to_build(nothing),
        }],
    }
}

fn site_notes(site: &BuiltSite) -> Vec<BuildNote> {
    let untitled = site.untitled.then(no_title);
    let large = site.large_search_documents.map(large_search_documents);
    untitled.into_iter().chain(large).collect()
}

fn no_title() -> BuildNote {
    BuildNote {
        kind: NoteKind::NoTitle,
        message: fl!(CATALOGUE, "no-title"),
    }
}

fn large_search_documents(bytes: usize) -> BuildNote {
    // Rounded up, so that a size above 5 MB never reads as 5.0.
    let size = format!("{:.1}", (bytes as f64 / 100_000.0).ceil() / 10.0);
    BuildNote {
        kind: NoteKind::LargeSearchDocuments,
        message: fl!(CATALOGUE, "search-documents-large", size = size.as_str()),
    }
}

/// Why a build that read no feature file wrote nothing, naming the list of `--only` when it was
/// given, or else the input folder.
pub(super) fn nothing_to_build(nothing: &NothingToBuild) -> String {
    match nothing {
        NothingToBuild::InputHoldsNone { input } => {
            fl!(CATALOGUE, "nothing-to-build", input = shown(input))
        }
        NothingToBuild::ListNamesNone { list } => fl!(
            CATALOGUE,
            "nothing-to-build-only",
            list = shown(&list.as_given())
        ),
        NothingToBuild::ListNamesNoPath { list } => fl!(
            CATALOGUE,
            "nothing-to-build-no-path",
            list = shown(&list.as_given())
        ),
    }
}

/// That the site `ghg serve --watch` serves is empty, since no feature file was read, naming the
/// list of `--only` when it was given, or else the input folder.
pub(super) fn empty_site(nothing: &NothingToBuild) -> String {
    match nothing {
        NothingToBuild::InputHoldsNone { input } => {
            fl!(CATALOGUE, "empty-site", input = shown(input))
        }
        NothingToBuild::ListNamesNone { list } => {
            fl!(CATALOGUE, "empty-site-only", list = shown(&list.as_given()))
        }
        NothingToBuild::ListNamesNoPath { list } => fl!(
            CATALOGUE,
            "empty-site-no-path",
            list = shown(&list.as_given())
        ),
    }
}

fn shown(path: &Path) -> String {
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::entities::PathList;

    fn list() -> PathList {
        PathList::File("changed.txt".into())
    }

    #[rstest]
    // Rules 204, 208, 229 and 230: each names the input folder, or the list as it was given.
    #[case::no_feature_file(
        NothingToBuild::InputHoldsNone { input: "specs".into() },
        "specs: the input folder holds no feature file, so no site was built.",
        "specs: the input folder holds no feature file, so the site served shows none."
    )]
    #[case::no_listed_feature_file(
        NothingToBuild::ListNamesNone { list: list() },
        "changed.txt: names no feature file of the input, so no site was built.",
        "changed.txt: names no feature file of the input, so the site served shows none."
    )]
    #[case::no_listed_path(
        NothingToBuild::ListNamesNoPath { list: PathList::StandardInput },
        "-: names no path, so there is nothing to build.",
        "-: names no path, so the site served shows none."
    )]
    fn nothing_read_is_worded_for_a_build_and_for_the_site_served(
        #[case] nothing: NothingToBuild,
        #[case] build: &str,
        #[case] served: &str,
    ) {
        assert_eq!(nothing_to_build(&nothing), build);
        assert_eq!(empty_site(&nothing), served);
    }
}
