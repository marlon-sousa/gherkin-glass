//! Error: why a feature file could not be parsed, as the feature parser port reports it.

use thiserror::Error;

use super::InputErrorKind;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
/// Its display is what is wrong, without the line.
#[error("{kind}")]
pub(crate) struct ParseError {
    pub(crate) line: Option<usize>,
    pub(crate) kind: InputErrorKind,
}
