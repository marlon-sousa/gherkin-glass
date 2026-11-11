//! Entity: why a feature file could not be parsed, as the feature parser port reports it.

use thiserror::Error;

use crate::entities::ProblemKind;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{kind}")]
pub(crate) struct ParseFailure {
    pub(crate) line: Option<usize>,
    pub(crate) kind: ProblemKind,
}
