//! Entity: how a build is run, from the command line over the configuration file over the defaults.

use std::path::PathBuf;

use crate::entities::{PageOptions, Unnamed, VersionName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub input: PathBuf,
    pub output: PathBuf,
    pub unnamed: Unnamed,
    pub pages: PageOptions,
    /// `None` when the build is not versioned.
    pub version: Option<VersionName>,
    /// Whether an earlier build of the site, or of the version, is replaced.
    pub replace: bool,
}
