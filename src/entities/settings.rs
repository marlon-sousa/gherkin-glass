//! Entity: how a build is run, from the command line over the configuration file over the defaults.

use std::path::PathBuf;

use crate::entities::{PageOptions, Unnamed};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub input: PathBuf,
    pub output: PathBuf,
    pub unnamed: Unnamed,
    pub pages: PageOptions,
use crate::entities::{Accents, Unnamed};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Settings {
    pub(crate) input: PathBuf,
    pub(crate) unnamed: Unnamed,
    pub(crate) accents: Accents,
}
