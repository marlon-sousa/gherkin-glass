//! Entity: what a configuration file sets, each value as the file writes it.

use std::path::PathBuf;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Config {
    /// Relative to the configuration file's folder; `None` when the file does not set it.
    pub(crate) input: Option<PathBuf>,
}
