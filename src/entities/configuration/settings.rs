//! Entity: how a build is run, from the command line over the configuration file over the defaults.

use std::path::PathBuf;

use crate::entities::{Destination, KeptInOutput, PageOptions, PathList, Unnamed, VersionName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Settings {
    pub(crate) input: PathBuf,
    pub(crate) output: PathBuf,
    pub(crate) destination: Destination,
    pub(crate) unnamed: Unnamed,
    pub(crate) pages: PageOptions,
    /// `None` when the build is not versioned.
    pub(crate) version: Option<VersionName>,
    /// Whether an earlier build of the site, or of the version, is replaced.
    pub(crate) replace: bool,
    /// The front page's template; `None` for ghg's own.
    pub(crate) front_page: Option<PathBuf>,
    /// Where `--only` reads its list of feature files from; `None` when the site shows every
    /// feature file.
    pub(crate) only: Option<PathList>,
    pub(crate) keep_in_output: KeptInOutput,
}
