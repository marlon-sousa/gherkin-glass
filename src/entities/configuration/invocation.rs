//! Entity: what the command line asks for.

use std::path::PathBuf;

use crate::entities::{Destination, VersionName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Invocation {
    Help,
    Version,
    Build(BuildOptions),
    Serve(ServeOptions),
}

/// The options of `ghg build`, each `None` when not given.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct BuildOptions {
    pub(crate) config: Option<PathBuf>,
    pub(crate) input: Option<PathBuf>,
    pub(crate) version_name: Option<VersionName>,
    pub(crate) output: Option<PathBuf>,
    pub(crate) title: Option<String>,
}

/// The options of `ghg serve`: those of `ghg build`, and how it serves.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ServeOptions {
    pub(crate) build: BuildOptions,
    pub(crate) host: Option<String>,
    pub(crate) port: Option<u16>,
    pub(crate) mode: ServeMode,
}

/// What `ghg serve` serves.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum ServeMode {
    /// The output folder as it is, under `--no-build`.
    Folder,
    /// The site built into the output folder first.
    #[default]
    Build,
    /// The site built in memory and built again on every change, under `--watch`.
    Watch,
}

impl ServeMode {
    /// Whether the site is built again on every change.
    pub(crate) fn watches(self) -> bool {
        self == ServeMode::Watch
    }

    pub(crate) fn destination(self) -> Destination {
        match self {
            ServeMode::Watch => Destination::Memory,
            ServeMode::Folder | ServeMode::Build => Destination::Folder,
        }
    }
}
