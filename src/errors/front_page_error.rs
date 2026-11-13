//! Error: what stops a build writing its front page, said of the front page's template and of its line when there is one.

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::PathBuf;

use i18n_embed_fl::fl;
use thiserror::Error;

use super::located;
use crate::policies::CATALOGUE;

/// Its display is the template's file, as the settings give it, its line when there is one,
/// then what is wrong.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) struct FrontPageError {
    pub(crate) file: PathBuf,
    /// Counted from 1; `None` for what is wrong with the whole file.
    pub(crate) line: Option<usize>,
    pub(crate) kind: FrontPageErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum FrontPageErrorKind {
    /// `reason` is the operating system's.
    #[error(fmt = Self::cannot_be_read)]
    CannotBeRead { reason: String },
    #[error(fmt = Self::no_title)]
    NoTitle,
    /// A level-one heading after the first.
    #[error(fmt = Self::second_title)]
    SecondTitle,
    #[error(fmt = Self::no_marker)]
    NoMarker,
    /// A marker after the first.
    #[error(fmt = Self::second_marker)]
    SecondMarker,
    /// A marker inside a list, a quote or a table, or within a line of text.
    #[error(fmt = Self::marker_not_alone)]
    MarkerNotAlone,
    /// A marker after a level 5 or level 6 heading, below which ghg's own headings cannot go.
    #[error(fmt = Self::marker_too_deep)]
    MarkerTooDeep,
    #[error(fmt = Self::raw_html)]
    RawHtml,
}

impl Display for FrontPageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        located(
            f,
            &self.file.to_string_lossy(),
            self.line,
            &self.kind.to_string(),
        )
    }
}

impl FrontPageErrorKind {
    fn cannot_be_read(reason: &str, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(
            CATALOGUE,
            "front-page-cannot-be-read",
            reason = reason
        ))
    }

    fn no_title(f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "front-page-no-title"))
    }

    fn second_title(f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "front-page-second-title"))
    }

    fn no_marker(f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "front-page-no-marker"))
    }

    fn second_marker(f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "front-page-second-marker"))
    }

    fn marker_not_alone(f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "front-page-marker-not-alone"))
    }

    fn marker_too_deep(f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "front-page-marker-too-deep"))
    }

    fn raw_html(f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&fl!(CATALOGUE, "front-page-raw-html"))
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::the_whole_file(
        None,
        FrontPageErrorKind::CannotBeRead { reason: "denied".into() },
        "docs/front.md: cannot be read: denied"
    )]
    #[case::a_line_of_it(
        Some(3),
        FrontPageErrorKind::RawHtml,
        "docs/front.md:3: raw HTML cannot be written in a front page; only the marker <!-- ghg:contents --> and other comments can."
    )]
    fn a_front_page_error_names_the_template_and_its_line(
        #[case] line: Option<usize>,
        #[case] kind: FrontPageErrorKind,
        #[case] expected: &str,
    ) {
        let error = FrontPageError {
            file: PathBuf::from("docs/front.md"),
            line,
            kind,
        };
        assert_eq!(error.to_string(), expected);
    }
}
