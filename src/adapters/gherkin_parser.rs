//! Adapter: the feature parser port over the `gherkin` crate.

use std::fs::read_to_string;
use std::io::Error as IoError;
use std::path::Path;

use gherkin::{EnvError, Feature, GherkinEnv, ParseFileError};

use crate::entities::{ParseFailure, ProblemKind, RawFeature};
use crate::ports::FeatureParser;

#[derive(Debug, Default, Clone, Copy)]
pub struct GherkinParser;

impl FeatureParser for GherkinParser {
    fn parse(&self, path: &Path) -> Result<RawFeature, ParseFailure> {
        let cannot_be_read = |error: IoError| ParseFailure {
            line: None,
            kind: ProblemKind::CannotBeRead {
                reason: error.to_string(),
            },
        };
        let source = read_to_string(path).map_err(cannot_be_read)?;
        // `parse_path` reads the file again, but only it returns the crate's
        // messages. `GherkinEnv` holds `RefCell`s, so each parse needs its own.
        let feature =
            Feature::parse_path(path, GherkinEnv::default()).map_err(|error| match error {
                ParseFileError::Reading { source, .. } => cannot_be_read(source),
                ParseFileError::Parsing {
                    error,
                    source: parse_error,
                    ..
                } => ParseFailure {
                    line: parser_line(&parse_error.to_string())
                        .map(|line| line.min(source.lines().count().max(1))),
                    kind: match error {
                        // The crate keeps the line break after an unknown keyword.
                        Some(EnvError::UnknownKeyword(keyword)) => ProblemKind::Parser {
                            message: format!("Unknown keyword: '{}'.", keyword.trim_end()),
                        },
                        Some(error) => ProblemKind::Parser {
                            message: error.to_string(),
                        },
                        None => ProblemKind::Syntax,
                    },
                },
            })?;
        Ok(RawFeature { feature, source })
    }
}

/// The crate gives the line only in its message, `Error at <line>:<column>: ...`,
/// and gives an error at the end of the file the line after the last.
fn parser_line(message: &str) -> Option<usize> {
    let rest = message.strip_prefix("Error at ")?;
    rest.split(':').next()?.parse().ok()
}
