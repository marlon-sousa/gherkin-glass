//! Error: one line of the report of what stops a build, the path, its line when there is one, and what is wrong, which a person reads and a program takes apart.

use std::fmt::{Display, Formatter, Result as FmtResult};

use i18n_embed_fl::fl;
use thiserror::Error;

use crate::policies::CATALOGUE;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) struct LocatedError {
    /// As the report shows it: relative to the input folder for the input's problems, and as it
    /// was given for the rest.
    pub(crate) path: String,
    pub(crate) line: Option<usize>,
    pub(crate) message: String,
}

/// `message` said of `path`, and of its line when there is one. The line number is passed as
/// text, so that Fluent never groups its digits.
impl Display for LocatedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let LocatedError {
            path,
            line,
            message,
        } = self;
        let text = match line {
            Some(line) => fl!(
                CATALOGUE,
                "problem-with-line",
                path = path.as_str(),
                line = line.to_string(),
                message = message.as_str()
            ),
            None => fl!(
                CATALOGUE,
                "problem",
                path = path.as_str(),
                message = message.as_str()
            ),
        };
        f.write_str(&text)
    }
}
