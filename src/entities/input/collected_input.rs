//! Entity: what collecting the input found, the paths of the files to read and the problems met.

use std::path::PathBuf;

use crate::entities::Problem;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CollectedInput {
    /// Sorted.
    pub(crate) paths: Vec<PathBuf>,
    pub(crate) problems: Vec<Problem>,
}
