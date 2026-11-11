//! Error: what stops a build, a versioned one included.

use thiserror::Error;

use super::{InputErrors, OutputError};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum BuildError {
    /// The input's errors, or a file of the site that cannot be written.
    #[error(transparent)]
    Input(#[from] InputErrors),
    #[error(transparent)]
    Output(#[from] OutputError),
}
