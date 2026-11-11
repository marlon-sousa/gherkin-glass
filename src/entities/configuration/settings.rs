//! Entity: how a build is run, from the command line over the configuration file over the defaults.

use std::path::PathBuf;

use crate::entities::{PageOptions, Unnamed, VersionName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Settings {
    pub(crate) input: PathBuf,
    pub(crate) output: PathBuf,
    pub(crate) unnamed: Unnamed,
    pub(crate) pages: PageOptions,
    /// `None` when the build is not versioned.
    pub(crate) version: Option<VersionName>,
    /// Whether an earlier build of the site, or of the version, is replaced.
    pub(crate) replace: bool,
}
