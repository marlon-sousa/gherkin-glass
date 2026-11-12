//! Entity: what collecting the input found, the paths of the files to read and the errors met.

use std::path::PathBuf;

use crate::errors::InputError;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CollectedInput {
    /// Sorted.
    pub(crate) paths: Vec<PathBuf>,
    pub(crate) errors: Vec<InputError>,
}
