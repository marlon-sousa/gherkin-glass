//! Error: what stops a build, a versioned one included, from its configuration to its output folder.

use thiserror::Error;

use super::{ConfigErrors, InputErrors, OutputError};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum BuildError {
    #[error(transparent)]
    Config(ConfigErrors),
    #[error(transparent)]
    Input(#[from] InputErrors),
    #[error(transparent)]
    Output(#[from] OutputError),
}
