//! Entity: how a build is run, from the command line over the configuration file over the defaults.

use std::path::PathBuf;

use crate::entities::{BuildTarget, Destination, KeptInOutput, PageOptions, PathList, Unnamed};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Settings {
    pub(crate) input: PathBuf,
    pub(crate) output: PathBuf,
    pub(crate) destination: Destination,
    pub(crate) unnamed: Unnamed,
    pub(crate) pages: PageOptions,
    /// What the command writes or removes.
    pub(crate) target: BuildTarget,
    /// Whether an earlier build of the site, or of the version, is replaced; a preview always is.
    pub(crate) replace: bool,
    /// The front page's template; `None` for ghg's own.
    pub(crate) front_page: Option<PathBuf>,
    /// Where `--only` reads its list of feature files from; `None` when the site shows every
    /// feature file.
    pub(crate) only: Option<PathList>,
    pub(crate) keep_in_output: KeptInOutput,
}
