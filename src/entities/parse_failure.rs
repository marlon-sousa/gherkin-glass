//! Entity: why a feature file could not be parsed, as the feature parser port reports it.

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct ParseFailure {
    pub line: Option<usize>,
    pub message: String,
}
