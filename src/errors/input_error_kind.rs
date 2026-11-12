//! Error: what is wrong with a thing of the input, apart from where it is.

use std::fmt::{Formatter, Result as FmtResult};

use i18n_embed_fl::fl;
use thiserror::Error;

use crate::policies::CATALOGUE;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Error)]
pub(crate) enum InputErrorKind {
    /// The input folder itself does not exist.
    #[error(fmt = Self::input_missing)]
    InputMissing,
    #[error(fmt = Self::cannot_be_listed)]
    CannotBeListed { reason: String },
    #[error(fmt = Self::cannot_be_read)]
    CannotBeRead { reason: String },
    /// `reason` is the `ignore` crate's.
    #[error("{reason}")]
    InvalidRule { reason: String },
    /// `message` is the `gherkin` crate's.
    #[error("{message}")]
    Parser { message: String },
    /// The parser named only what it expected, in an order that changes from run to run.
    #[error(fmt = Self::syntax)]
    Syntax,
    /// The file changed between its parse and the read of its source.
    #[error(fmt = Self::changed)]
    Changed,
    /// Only while `allow_unnamed` is `false`.
    #[error(fmt = Self::unnamed)]
    Unnamed { block: UnnamedBlock },
    /// `page` is the address both files would have, and `first` the path,
    /// shown as an error's is, of the file that has it.
    #[error(fmt = Self::same_page)]
    SamePage { page: String, first: String },
    /// A folder whose page's address differs from an earlier folder's only
    /// in case; `page` is its own, and `first` that folder's path.
    #[error(fmt = Self::same_folder_page)]
    SameFolderPage { page: String, first: String },
    /// `page` is the address of the page of step sentences.
    #[error(fmt = Self::reserved_page)]
    ReservedPage { page: String },
    /// `page` is the file's own page, at the path of a folder's, and `folder`
    /// that folder's path below the input folder, `None` for the input
    /// folder's, the home page.
    #[error(fmt = Self::same_as_folder_page)]
    SameAsFolderPage {
        page: String,
        folder: Option<String>,
    },
    /// `page` is the file's page, in the folder that holds the tag pages.
    #[error(fmt = Self::in_tags_folder)]
    InTagsFolder { page: String },
}

/// The blocks that need a name while `allow_unnamed` is `false`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum UnnamedBlock {
    Feature,
    Rule,
    Scenario,
    /// A scenario with examples.
    ScenarioOutline,
}

impl InputErrorKind {
    fn input_missing(f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "input-missing"))
    }

    fn cannot_be_listed(reason: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "input-cannot-be-listed", reason = reason))
    }

    fn cannot_be_read(reason: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "input-cannot-be-read", reason = reason))
    }

    fn syntax(f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "input-syntax-error"))
    }

    fn changed(f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "input-changed"))
    }

    fn unnamed(block: &UnnamedBlock, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&match block {
            UnnamedBlock::Feature => fl!(CATALOGUE, "unnamed-feature"),
            UnnamedBlock::Rule => fl!(CATALOGUE, "unnamed-rule"),
            UnnamedBlock::Scenario => fl!(CATALOGUE, "unnamed-scenario"),
            UnnamedBlock::ScenarioOutline => fl!(CATALOGUE, "unnamed-scenario-outline"),
        })
    }

    fn same_page(page: &str, first: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "same-page", page = page, first = first))
    }

    fn same_folder_page(page: &str, first: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(
            CATALOGUE,
            "same-folder-page",
            page = page,
            first = first
        ))
    }

    fn reserved_page(page: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "reserved-page", page = page))
    }

    fn same_as_folder_page(
        page: &str,
        folder: &Option<String>,
        f: &mut Formatter<'_>,
    ) -> FmtResult {
        f.write_str(&match folder {
            Some(folder) => fl!(
                CATALOGUE,
                "same-as-folder-page",
                page = page,
                folder = folder.as_str()
            ),
            None => fl!(CATALOGUE, "same-as-home-page", page = page),
        })
    }

    fn in_tags_folder(page: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "in-tags-folder", page = page))
    }
}
