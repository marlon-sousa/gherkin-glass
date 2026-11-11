//! Entity: what stops a build, a versioned one included.

use thiserror::Error;

use crate::entities::{OutputProblem, Problems};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BuildFailure {
    /// The input's problems, or a file of the site that cannot be written.
    #[error("{0}")]
    Problems(#[from] Problems),
    #[error("{0}")]
    Output(#[from] OutputProblem),
}
