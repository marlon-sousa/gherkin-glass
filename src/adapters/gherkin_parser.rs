//! Adapter: the feature parser port over the `gherkin` crate, which reads the file itself.

use std::fs;
use std::path::Path;

use gherkin::{EnvError, Feature, GherkinEnv, ParseFileError};

use crate::entities::ParseFailure;
use crate::ports::FeatureParser;

#[derive(Debug, Default, Clone, Copy)]
pub struct GherkinParser;

impl FeatureParser for GherkinParser {
    fn parse(&self, path: &Path) -> Result<Feature, ParseFailure> {
        // `GherkinEnv` holds `RefCell`s, so each parse needs its own to run on
        // its own thread.
        Feature::parse_path(path, GherkinEnv::default()).map_err(|error| match error {
            ParseFileError::Reading { source, .. } => ParseFailure {
                line: None,
                message: format!("cannot be read: {source}"),
            },
            ParseFileError::Parsing { error, source, .. } => ParseFailure {
                line: parser_line(&source.to_string()).map(|line| within_file(path, line)),
                message: match error {
                    // The crate keeps the line break after an unknown keyword.
                    Some(EnvError::UnknownKeyword(keyword)) => {
                        format!("Unknown keyword: '{}'.", keyword.trim_end())
                    }
                    Some(error) => error.to_string(),
                    // The crate's own message lists what it expected, in an
                    // order that changes from run to run.
                    None => "Syntax error.".to_string(),
                },
            },
        })
    }
}

/// The crate gives the line only in its message, `Error at <line>:<column>: ...`.
fn parser_line(message: &str) -> Option<usize> {
    let rest = message.strip_prefix("Error at ")?;
    rest.split(':').next()?.parse().ok()
}

/// The crate reports an error at the end of the file on the line after the last.
fn within_file(path: &Path, line: usize) -> usize {
    match fs::read_to_string(path) {
        Ok(source) => line.min(source.lines().count().max(1)),
        Err(_) => line,
    }
}
