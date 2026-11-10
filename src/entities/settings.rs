//! Entity: how a build is run, from the command line over the configuration file over the defaults.

use std::path::PathBuf;

use crate::entities::Unnamed;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Settings {
    pub(crate) input: PathBuf,
    pub(crate) unnamed: Unnamed,
}
