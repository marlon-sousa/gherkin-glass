//! Entity: the file of ignore rules a folder of the input may hold, by its name, and its rules as a listed folder of the input tree keeps them.

use std::sync::Arc;

use ignore::gitignore::Gitignore;

use crate::errors::InputError;

pub(crate) const IGNORE_FILE: &str = ".ghg-ignore";

/// The rules of a folder's `.ghg-ignore`, kept so that a later walk that lists the folder again
/// without its `.ghg-ignore` having changed takes them in without reading it again.
#[derive(Debug, Clone, Default)]
pub(crate) struct IgnoreRules {
    /// The valid rules, none when the folder has no `.ghg-ignore` or it could not be read,
    /// shared with the trees made from this one.
    pub(crate) rules: Option<Arc<Gitignore>>,
    /// The error met reading the `.ghg-ignore`, or those of its rules that are not valid.
    pub(crate) errors: Vec<InputError>,
}
