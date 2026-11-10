//! Entity: why a feature file could not be parsed, as the feature parser port reports it.

use thiserror::Error;

use crate::entities::ProblemKind;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{kind}")]
pub struct ParseFailure {
    pub line: Option<usize>,
    pub kind: ProblemKind,
}
