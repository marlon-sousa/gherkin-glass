//! Entity: a file whose page clashes with another page, or a folder whose path differs from another folder's only in case, with its error.

use std::path::PathBuf;

use crate::errors::InputError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PageClash {
    /// Of the file or the folder, as the input tree holds it.
    pub(crate) path: PathBuf,
    pub(crate) error: InputError,
}
