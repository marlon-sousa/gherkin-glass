//! Entity: what collecting the input found, the paths of the files to read and the problems met.

use std::path::PathBuf;

use crate::entities::Problem;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CollectedInput {
    /// Sorted.
    pub paths: Vec<PathBuf>,
    pub problems: Vec<Problem>,
}
