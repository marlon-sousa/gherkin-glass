//! Error: what stops a build, a versioned one included, from its configuration to its output folder and its front page.

use thiserror::Error;

use super::{ConfigErrors, FrontPageError, InputErrors, LocatedError, OutputError};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum BuildError {
    #[error(transparent)]
    Config(ConfigErrors),
    #[error(transparent)]
    Input(#[from] InputErrors),
    #[error(transparent)]
    Output(#[from] OutputError),
    #[error(transparent)]
    FrontPage(#[from] FrontPageError),
}

impl BuildError {
    /// Each line of the report, in its order.
    pub(crate) fn problems(&self) -> Vec<LocatedError> {
        match self {
            BuildError::Config(errors) => errors.problems(),
            BuildError::Input(errors) => errors.problems(),
            BuildError::Output(error) => vec![error.problem()],
            BuildError::FrontPage(error) => vec![error.problem()],
        }
    }
}
