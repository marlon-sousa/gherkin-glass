//! Adapter: the feature parser port over the `gherkin` crate.

use std::fs::read_to_string;
use std::io::Error as IoError;
use std::path::Path;

use gherkin::{EnvError, Feature, GherkinEnv, ParseFileError};

use crate::entities::RawFeature;
use crate::errors::{InputErrorKind, ParseError};
use crate::ports::FeatureParser;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct GherkinCrate;

impl FeatureParser for GherkinCrate {
    fn parse(&self, path: &Path) -> Result<RawFeature, ParseError> {
        let source = read_to_string(path).map_err(cannot_be_read)?;
        // `parse_path` reads the file again, but only it returns the crate's
        // messages. `GherkinEnv` holds `RefCell`s, so each parse needs its own.
        let feature = Feature::parse_path(path, GherkinEnv::default())
            .map_err(|error| parse_error(error, &source))?;
        Ok(RawFeature { feature, source })
    }
}

fn cannot_be_read(error: IoError) -> ParseError {
    ParseError {
        line: None,
        kind: InputErrorKind::CannotBeRead {
            reason: error.to_string(),
        },
    }
}

fn parse_error(error: ParseFileError, source: &str) -> ParseError {
    match error {
        ParseFileError::Reading { source, .. } => cannot_be_read(source),
        ParseFileError::Parsing {
            error,
            source: parse_error,
            ..
        } => ParseError {
            line: parser_line(&parse_error.to_string()).map(|line| line_in(source, line)),
            kind: parser_error(error),
        },
    }
}

/// The crate gives the line only in its message, `Error at <line>:<column>: ...`.
fn parser_line(message: &str) -> Option<usize> {
    let rest = message.strip_prefix("Error at ")?;
    rest.split(':').next()?.parse().ok()
}

/// The crate gives an error at the end of the file the line after the last.
fn line_in(source: &str, line: usize) -> usize {
    line.min(source.lines().count().max(1))
}

fn parser_error(error: Option<EnvError>) -> InputErrorKind {
    match error {
        // The crate keeps the line break after an unknown keyword.
        Some(EnvError::UnknownKeyword(keyword)) => InputErrorKind::Parser {
            message: EnvError::UnknownKeyword(keyword.trim_end().to_string()).to_string(),
        },
        Some(error) => InputErrorKind::Parser {
            message: error.to_string(),
        },
        None => InputErrorKind::Syntax,
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::with_its_column("Error at 3:7: unexpected", Some(3))]
    #[case::without_one("Something else", None)]
    fn the_line_is_read_from_the_crate_s_message(
        #[case] message: &str,
        #[case] line: Option<usize>,
    ) {
        assert_eq!(parser_line(message), line);
    }

    #[rstest]
    #[case::inside(2, 2)]
    #[case::after_the_last(4, 3)]
    fn an_error_after_the_last_line_is_on_the_last(#[case] given: usize, #[case] line: usize) {
        assert_eq!(line_in("a\nb\nc\n", given), line);
    }

    #[test]
    fn an_unknown_keyword_is_named_without_its_line_break() {
        assert_eq!(
            parser_error(Some(EnvError::UnknownKeyword("Wen \n".to_string()))),
            InputErrorKind::Parser {
                message: "Unknown keyword: 'Wen'.".to_string()
            }
        );
    }
}
